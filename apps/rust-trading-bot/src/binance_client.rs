use crate::exchange_trait::*;
use anyhow::Result;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use log::{error, info, warn};
use reqwest;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct AccountInformation {
    pub totalWalletBalance: String,
    pub availableBalance: String,
    pub totalUnrealizedProfit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct IncomeRecord {
    pub symbol: String,
    pub incomeType: String,
    pub income: String, // 金额,字符串格式
    pub time: i64,      // 毫秒时间戳
}

#[derive(Debug, Clone, Deserialize)]
#[allow(non_snake_case)]
pub struct UserTrade {
    pub symbol: String,
    pub id: i64,
    pub orderId: i64,
    pub side: String, // "BUY" or "SELL"
    pub price: String,
    pub qty: String,
    pub quoteQty: String, // 名义价值 = price * qty
    pub commission: String,
    pub commissionAsset: String,
    pub time: i64,
    pub positionSide: String, // "LONG" or "SHORT"
    pub realizedPnl: String,
}

/// 币种历史表现统计
#[derive(Debug, Clone)]
pub struct SymbolPerformance {
    pub symbol: String,
    pub trade_count: usize,
    pub win_count: usize,
    pub loss_count: usize,
    pub total_pnl: f64,
    pub total_margin: f64,
    pub margin_loss_rate: f64, // 保证金收益率 (%)
    pub win_rate: f64,         // 胜率 (%)
}

/// 风险等级
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    High,   // 保证金亏损率 < -15%
    Medium, // 保证金亏损率 -15% ~ -10%
    Low,    // 保证金亏损率 -10% ~ -5%
    Normal, // 保证金亏损率 > -5%
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct PositionRisk {
    symbol: String,
    positionAmt: String,
    entryPrice: String,
    markPrice: String,
    unRealizedProfit: String,
    leverage: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct SymbolInfo {
    symbol: String,
    quantityPrecision: i32,
    pricePrecision: i32,
    filters: Vec<FilterInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct FilterInfo {
    filterType: String,
    stepSize: Option<String>,
    minQty: Option<String>,
    maxQty: Option<String>,
    tickSize: Option<String>, // PRICE_FILTER的价格步长
    minPrice: Option<String>,
    maxPrice: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BinanceClient {
    api_key: String,
    secret_key: String,
    base_url: String,      // FAPI endpoint (fapi.binance.com)
    papi_base_url: String, // Portfolio Margin API endpoint (papi.binance.com)
    // 缓存每个交易对的交易规则，减少 exchangeInfo 请求
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let (base_url, papi_base_url) = if testnet {
            // Testnet uses same domain for both fapi and papi
            (
                "https://testnet.binancefuture.com".to_string(),
                "https://testnet.binancefuture.com".to_string(),
            )
        } else {
            // Mainnet has separate domains
            (
                "https://fapi.binance.com".to_string(),
                "https://papi.binance.com".to_string(),
            )
        };

        Self {
            api_key,
            secret_key,
            base_url,
            papi_base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建强制使用 IPv4 的 HTTP 客户端
    fn create_ipv4_client(&self) -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder()
            .local_address(Some(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)))
            .build()?)
    }

    fn sign_request(&self, query: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(query.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    pub async fn get_account_info(&self) -> Result<AccountInformation> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v2/account?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("获取账户信息失败: {}", body);
            return Err(anyhow::anyhow!("API错误 ({}): {}", status, body));
        }

        let account: AccountInformation = serde_json::from_str(&body)?;
        info!("账户余额: {} USDT", account.totalWalletBalance);
        info!("未实现盈亏: {} USDT", account.totalUnrealizedProfit);
        Ok(account)
    }

    pub async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side_position: bool,
    ) -> Result<()> {
        // 统一设置模式与杠杆
        // 忽略“无需变更”的错误
        let _ = self.set_position_mode(dual_side_position).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        // 使用当前价格略微加价，提升限价单成交概率
        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 1.001;
        let position_side = "LONG";

        let _order_id = self
            .limit_order(symbol, quantity, "BUY", limit_price, Some(position_side))
            .await?;

        info!(
            "✅ 开多成功: {} x{} 杠杆, 数量: {}, 限价: ${:.4}",
            symbol, leverage, quantity, limit_price
        );
        Ok(())
    }

    pub async fn open_short(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side_position: bool,
    ) -> Result<()> {
        // 统一设置模式与杠杆
        let _ = self.set_position_mode(dual_side_position).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        // 使用当前价格略微减价，提升限价单成交概率
        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 0.999;
        let position_side = "SHORT";

        let _order_id = self
            .limit_order(symbol, quantity, "SELL", limit_price, Some(position_side))
            .await?;

        info!(
            "✅ 开空成功: {} x{} 杠杆, 数量: {}, 限价: ${:.4}",
            symbol, leverage, quantity, limit_price
        );
        Ok(())
    }

    pub async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!(
            "symbol={}&marginType={}&timestamp={}",
            symbol, margin_type, timestamp
        );
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/marginType?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            let body_lower = body.to_lowercase();
            // 统一账户(Portfolio Margin)强制全仓,调用set_margin_type会返回-2015权限错误
            // 标准合约账户如果已是目标模式会返回"no need to change"
            // 两种情况都应该忽略错误继续交易
            if body_lower.contains("no need to change")
                || body_lower.contains("does not need to be adjusted")
                || body_lower.contains("-2015")
                || body_lower.contains("invalid api-key")
            {
                warn!("⚠️  设置margin_type被跳过 (可能是统一账户): {}", body);
                return Ok(());
            }
            error!("设置仓位模式失败: {}", body);
            return Err(anyhow::anyhow!("设置仓位模式失败: {}", body));
        }

        info!("✅ {} 仓位模式已切换为 {}", symbol, margin_type);
        Ok(())
    }

    pub async fn set_multi_assets_margin(&self, enabled: bool) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("multiAssetsMargin={}&timestamp={}", enabled, timestamp);
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/multiAssetsMargin?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            let body_lower = body.to_lowercase();
            if body_lower.contains("no need to change")
                || body_lower.contains("does not need to be adjusted")
            {
                info!(
                    "ℹ️  保证金模式已为 {}币种模式，无需改动",
                    if enabled { "多" } else { "单" }
                );
                return Ok(());
            }
            error!("设置保证金资产模式失败: {}", body);
            return Err(anyhow::anyhow!("设置保证金资产模式失败: {}", body));
        }

        info!(
            "✅ 已切换为 {}币种保证金模式",
            if enabled { "多" } else { "单" }
        );
        Ok(())
    }

    pub async fn close_position(&self, symbol: &str, side: &str, quantity: f64) -> Result<()> {
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };
        self.market_order(symbol, quantity, order_side).await?;
        info!("✅ 平仓成功: {} {} {}", symbol, side, quantity);
        Ok(())
    }

    async fn change_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!(
            "symbol={}&leverage={}&timestamp={}",
            symbol, leverage, timestamp
        );
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/leverage?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        Ok(())
    }

    pub async fn market_order(&self, symbol: &str, quantity: f64, side: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 获取交易规则与当前价格
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let current_price = self.get_current_price(symbol).await?;

        let min_notional = 21.0;

        // 名义金额兜底 ≥ 21 USDT
        let mut qty = quantity;
        if qty * current_price < min_notional {
            let adjusted = min_notional / current_price;
            info!(
                "⚙️  数量自动调整以满足最低名义金额{:.0}U: {:.6} -> {:.6}",
                min_notional, qty, adjusted
            );
            qty = adjusted;
        }

        // 按 stepSize 向下对齐
        let step = rules.step_size;
        let mut adjusted_quantity = (qty / step).floor() * step;

        // 确保不低于最小数量
        if adjusted_quantity < rules.min_qty {
            adjusted_quantity = rules.min_qty;
        }

        // 再次检查名义金额 ≥ 5 USDT，必要时提升并对齐
        if adjusted_quantity * current_price < min_notional {
            let needed_qty = min_notional / current_price;
            adjusted_quantity = (needed_qty / step).ceil() * step;
        }

        // 根据 quantity_precision 进行格式化
        let precision = rules.quantity_precision.max(0) as usize;
        let quantity_str = format!("{:.*}", precision, adjusted_quantity);

        // 根据方向确定 positionSide (做多LONG, 做空SHORT)
        let position_side = if side == "BUY" { "LONG" } else { "SHORT" };

        let query = format!(
            "symbol={}&side={}&type=MARKET&quantity={}&positionSide={}&timestamp={}",
            symbol, side, quantity_str, position_side, timestamp
        );
        let signature = self.sign_request(&query);

        // 优先使用 PAPI (Portfolio Margin API) for unified account
        let url = format!(
            "{}/papi/v1/um/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("订单失败: {}", body);
            return Err(anyhow::anyhow!("订单失败: {}", body));
        }

        Ok(())
    }

    pub async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        let url = format!("{}/fapi/v1/ticker/price?symbol={}", self.base_url, symbol);

        let client = self.create_ipv4_client()?;
        let response: serde_json::Value = client.get(&url).send().await?.json().await?;

        let price: f64 = response["price"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("价格解析失败"))?
            .parse()?;

        Ok(price)
    }

    pub async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        // 先查缓存
        if let Some(cached) = self.rules_cache.read().await.get(symbol).cloned() {
            return Ok(cached);
        }

        // 未命中则请求并写入缓存
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);
        let client = self.create_ipv4_client()?;
        let response: ExchangeInfo = client.get(&url).send().await?.json().await?;

        for symbol_info in response.symbols {
            if symbol_info.symbol == symbol {
                let mut step_size_val = None;
                let mut min_qty_val = None;
                let mut tick_size_val = None;

                for filter in &symbol_info.filters {
                    if filter.filterType == "LOT_SIZE" {
                        let step_size = filter
                            .stepSize
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("缺少stepSize信息"))?
                            .parse::<f64>()?;
                        let min_qty = filter
                            .minQty
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("缺少minQty信息"))?
                            .parse::<f64>()?;
                        step_size_val = Some(step_size);
                        min_qty_val = Some(min_qty);
                    }
                    if filter.filterType == "PRICE_FILTER" {
                        let tick_size = filter
                            .tickSize
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("缺少tickSize信息"))?
                            .parse::<f64>()?;
                        tick_size_val = Some(tick_size);
                    }
                }

                let rules = TradingRules {
                    step_size: step_size_val.ok_or_else(|| anyhow::anyhow!("缺少stepSize信息"))?,
                    min_qty: min_qty_val.ok_or_else(|| anyhow::anyhow!("缺少minQty信息"))?,
                    quantity_precision: symbol_info.quantityPrecision,
                    price_precision: symbol_info.pricePrecision,
                    tick_size: tick_size_val.ok_or_else(|| anyhow::anyhow!("缺少tickSize信息"))?,
                };

                self.rules_cache
                    .write()
                    .await
                    .insert(symbol.to_string(), rules.clone());

                return Ok(rules);
            }
        }

        Err(anyhow::anyhow!("未找到交易对信息: {}", symbol))
    }

    /// 统一设置交易模式（单向/双向）与逐仓/全仓，并调整杠杆
    pub async fn ensure_trading_modes(
        &self,
        symbol: &str,
        leverage: u32,
        margin_type: &str,
        dual_side_position: bool,
    ) -> Result<()> {
        let _ = self.set_position_mode(dual_side_position).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;
        Ok(())
    }

    pub fn calculate_quantity_with_margin(
        &self,
        price: f64,
        margin: f64,
        leverage: u32,
        trading_rules: &TradingRules,
    ) -> Result<f64> {
        // 正确的计算逻辑：用指定保证金开杠杆仓位
        let notional_value = margin * leverage as f64;
        let raw_quantity = notional_value / price;

        // 根据stepSize调整数量（这是关键修复）
        let adjusted_quantity =
            (raw_quantity / trading_rules.step_size).floor() * trading_rules.step_size;

        // 检查最小数量限制
        if adjusted_quantity < trading_rules.min_qty {
            return Err(anyhow::anyhow!(
                "计算数量 {:.8} 小于最小数量限制 {:.8}",
                adjusted_quantity,
                trading_rules.min_qty
            ));
        }

        info!("📊 数量计算详情:");
        info!("   原始数量: {:.8}", raw_quantity);
        info!(
            "   步长调整: {:.8} (stepSize: {:.8})",
            adjusted_quantity, trading_rules.step_size
        );
        info!("   最小数量: {:.8}", trading_rules.min_qty);
        info!("   精度位数: {}", trading_rules.quantity_precision);

        Ok(adjusted_quantity)
    }

    async fn set_position_mode(&self, dual_side_position: bool) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!(
            "dualSidePosition={}&timestamp={}",
            dual_side_position, timestamp
        );
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/positionSide/dual?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            let body_lower = body.to_lowercase();
            // 统一账户可能不支持设置持仓模式或已设置,忽略这些错误
            if body_lower.contains("no need to change")
                || body_lower.contains("not modified")
                || body_lower.contains("-2015")
                || body_lower.contains("invalid api-key")
            {
                warn!("⚠️  设置持仓模式被跳过 (可能是统一账户): {}", body);
                return Ok(());
            }
            error!("设置持仓模式失败: {}", body);
            return Err(anyhow::anyhow!("设置持仓模式失败: {}", body));
        }

        info!(
            "✅ 持仓模式设置成功: {}",
            if dual_side_position {
                "双向持仓"
            } else {
                "单向持仓"
            }
        );
        Ok(())
    }

    /// 设置止损单 (STOP_MARKET)
    pub async fn set_stop_loss(
        &self,
        symbol: &str,
        side: &str, // "LONG" or "SHORT" - 持仓方向
        _quantity: f64,
        stop_price: f64,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 平多仓用SELL,平空仓用BUY
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };
        let position_side = side; // PAPI 要求显式传入 LONG/SHORT

        // 获取交易规则以便获取精度信息
        let rules = self.get_symbol_trading_rules(symbol).await?;

        // 获取价格精度并调整止损价
        let price_precision = rules.price_precision.max(0) as usize;
        let stop_price_str = format!("{:.*}", price_precision, stop_price);

        // PAPI 条件单需要 workingType + positionSide + priceProtect 参数
        // 注意: 条件单不支持 reduceOnly 参数,positionSide 已经决定了平仓方向
        let query = format!(
            "symbol={}&side={}&strategyType=STOP_MARKET&stopPrice={}&positionSide={}&workingType=MARK_PRICE&priceProtect=true&timestamp={}",
            symbol, order_side, stop_price_str, position_side, timestamp
        );
        let signature = self.sign_request(&query);

        // 优先使用 PAPI (Portfolio Margin API) for unified account
        let url = format!(
            "{}/papi/v1/um/conditional/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("设置止损单失败: {}", body);
            return Err(anyhow::anyhow!("设置止损单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 止损单已设置: {} {} @ ${} (订单ID: {})",
            symbol, order_side, stop_price, order_id
        );
        Ok(order_id)
    }

    /// 设置止盈单 (TAKE_PROFIT_MARKET)
    pub async fn set_take_profit(
        &self,
        symbol: &str,
        side: &str, // "LONG" or "SHORT" - 持仓方向
        quantity: f64,
        stop_price: f64,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 平多仓用SELL,平空仓用BUY
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };

        // 获取交易规则并调整数量
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let precision = rules.quantity_precision.max(0) as usize;
        let quantity_str = format!("{:.*}", precision, quantity);

        // 获取价格精度并调整止盈价
        let price_precision = rules.price_precision.max(0) as usize;
        let stop_price_str = format!("{:.*}", price_precision, stop_price);

        // PAPI 条件单需要 workingType + positionSide + priceProtect 参数
        // 注意: 条件单不支持 reduceOnly 和 timeInForce 参数
        let position_side = side; // "LONG" or "SHORT"
        let query = format!(
            "symbol={}&side={}&strategyType=TAKE_PROFIT_MARKET&stopPrice={}&quantity={}&positionSide={}&workingType=MARK_PRICE&priceProtect=true&timestamp={}",
            symbol, order_side, stop_price_str, quantity_str, position_side, timestamp
        );
        let signature = self.sign_request(&query);

        // 优先使用 PAPI (Portfolio Margin API) for unified account
        let url = format!(
            "{}/papi/v1/um/conditional/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("设置止盈单失败: {}", body);
            return Err(anyhow::anyhow!("设置止盈单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 止盈单已设置: {} {} @ ${} (订单ID: {})",
            symbol, order_side, stop_price, order_id
        );
        Ok(order_id)
    }

    /// 设置限价止盈单 (LIMIT order for take profit)
    pub async fn set_limit_take_profit(
        &self,
        symbol: &str,
        side: &str, // "LONG" or "SHORT" - 持仓方向
        quantity: f64,
        limit_price: f64, // 限价价格
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 平多仓用SELL,平空仓用BUY
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };
        let position_side = side; // Hedge Mode 必须显式指明仓位方向

        // 获取交易规则
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        // 格式化数量和价格
        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let price_str = format!("{:.*}", price_precision, limit_price);

        let query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&positionSide={}&reduceOnly=true&timeInForce=GTC&timestamp={}",
            symbol, order_side, price_str, quantity_str, position_side, timestamp
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/papi/v1/um/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("设置限价止盈单失败: {}", body);
            return Err(anyhow::anyhow!("设置限价止盈单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 限价止盈单已设置: {} {} @ ${} (订单ID: {})",
            symbol, order_side, limit_price, order_id
        );
        Ok(order_id)
    }

    /// 通用限价单 (支持传入 BUY/SELL 以及可选 positionSide)
    pub async fn limit_order(
        &self,
        symbol: &str,
        quantity: f64,
        side: &str, // "BUY" or "SELL"
        limit_price: f64,
        position_side: Option<&str>,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        let rules = self.get_symbol_trading_rules(symbol).await?;

        // 先按 tick_size 对齐价格，避免提交非法价格
        let aligned_price = (limit_price / rules.tick_size).floor() * rules.tick_size;

        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let price_str = format!("{:.*}", price_precision, aligned_price);

        let mut query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, side, price_str, quantity_str, timestamp
        );

        if let Some(pos_side) = position_side {
            query = format!("{}&positionSide={}", query, pos_side);
        }

        let signature = self.sign_request(&query);

        let url = format!(
            "{}/papi/v1/um/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("设置限价单失败: {}", body);
            return Err(anyhow::anyhow!("设置限价单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 限价单已下单: {} {} @ ${} (订单ID: {})",
            symbol, side, aligned_price, order_id
        );
        Ok(order_id)
    }

    /// 下限价单 (通用限价单,可用于开仓或平仓)
    pub async fn set_limit_order(
        &self,
        symbol: &str,
        side: &str, // "BUY" or "SELL"
        quantity: f64,
        limit_price: f64,
        position_side: Option<&str>, // "LONG" or "SHORT", None for closing
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 获取交易规则
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        // 格式化数量和价格
        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let price_str = format!("{:.*}", price_precision, limit_price);

        // 构建查询参数
        let mut query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, side, price_str, quantity_str, timestamp
        );

        // 如果指定了持仓方向,添加 positionSide
        if let Some(pos_side) = position_side {
            query = format!("{}&positionSide={}", query, pos_side);
        }

        let signature = self.sign_request(&query);

        let url = format!(
            "{}/papi/v1/um/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("设置限价单失败: {}", body);
            return Err(anyhow::anyhow!("设置限价单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 限价单已设置: {} {} @ ${} (订单ID: {})",
            symbol, side, limit_price, order_id
        );
        Ok(order_id)
    }

    /// 取消订单
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );
        let signature = self.sign_request(&query);

        // 优先使用 PAPI (Portfolio Margin API) for unified account
        let url = format!(
            "{}/papi/v1/um/order?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            warn!("取消订单失败: {}", body);
            return Err(anyhow::anyhow!("取消订单失败: {}", body));
        }

        info!("✅ 订单已取消: {} (订单ID: {})", symbol, order_id);
        Ok(())
    }

    /// 获取指定时间范围内的已实现盈亏历史
    /// hours: 查询最近N小时的数据
    pub async fn get_income_history(&self, hours: u64) -> Result<Vec<IncomeRecord>> {
        let end_time = chrono::Utc::now().timestamp_millis();
        let start_time = end_time - (hours as i64 * 3600 * 1000);

        let query = format!(
            "startTime={}&endTime={}&incomeType=REALIZED_PNL&timestamp={}",
            start_time, end_time, end_time
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/papi/v1/um/income?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("获取收益历史失败: {}", body);
            return Err(anyhow::anyhow!("获取收益历史失败: {}", body));
        }

        let records: Vec<IncomeRecord> = response.json().await?;
        info!("📊 获取到 {} 条收益记录 (最近{}小时)", records.len(), hours);
        Ok(records)
    }

    /// 获取指定时间范围内的用户成交记录
    /// hours: 查询最近N小时的数据
    pub async fn get_user_trades(&self, hours: u64) -> Result<Vec<UserTrade>> {
        let end_time = chrono::Utc::now().timestamp_millis();
        let start_time = end_time - (hours as i64 * 3600 * 1000);

        let query = format!(
            "startTime={}&endTime={}&timestamp={}",
            start_time, end_time, end_time
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/papi/v1/um/userTrades?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            error!("获取成交记录失败: {}", body);
            return Err(anyhow::anyhow!("获取成交记录失败: {}", body));
        }

        let trades: Vec<UserTrade> = response.json().await?;
        info!("📊 获取到 {} 条成交记录 (最近{}小时)", trades.len(), hours);
        Ok(trades)
    }

    /// 获取币种历史表现统计
    pub async fn get_symbol_performance(
        &self,
        symbol: &str,
        hours: u64,
    ) -> Result<Option<SymbolPerformance>> {
        // 1. 获取收益历史
        let income_records = self.get_income_history(hours).await?;

        // 2. 获取成交记录
        let user_trades = self.get_user_trades(hours).await?;

        // 3. 过滤该币种的数据
        let symbol_incomes: Vec<_> = income_records
            .iter()
            .filter(|r| r.symbol == symbol)
            .collect();

        if symbol_incomes.is_empty() {
            return Ok(None); // 没有历史数据
        }

        // 4. 计算统计数据
        let mut total_pnl = 0.0;
        let mut win_count = 0;
        let mut loss_count = 0;

        for record in &symbol_incomes {
            let income: f64 = record.income.parse().unwrap_or(0.0);
            total_pnl += income;
            if income > 0.0 {
                win_count += 1;
            } else if income < 0.0 {
                loss_count += 1;
            }
        }

        // 5. 计算保证金
        let mut total_margin = 0.0;
        const DEFAULT_LEVERAGE: f64 = 10.0;

        for trade in &user_trades {
            if trade.symbol != symbol {
                continue;
            }

            let notional = trade.quoteQty.parse::<f64>().unwrap_or(0.0);
            let is_open = (trade.side == "BUY" && trade.positionSide == "LONG")
                || (trade.side == "SELL" && trade.positionSide == "SHORT");

            if is_open && notional > 0.0 {
                total_margin += notional / DEFAULT_LEVERAGE;
            }
        }

        // 6. 计算收益率和胜率
        let margin_loss_rate = if total_margin > 0.0 {
            (total_pnl / total_margin) * 100.0
        } else {
            0.0
        };

        let trade_count = symbol_incomes.len();
        let win_rate = if trade_count > 0 {
            (win_count as f64 / trade_count as f64) * 100.0
        } else {
            0.0
        };

        Ok(Some(SymbolPerformance {
            symbol: symbol.to_string(),
            trade_count,
            win_count,
            loss_count,
            total_pnl,
            total_margin,
            margin_loss_rate,
            win_rate,
        }))
    }

    /// 判断风险等级
    pub fn get_risk_level(perf: &SymbolPerformance) -> RiskLevel {
        if perf.margin_loss_rate < -15.0 {
            RiskLevel::High
        } else if perf.margin_loss_rate < -10.0 {
            RiskLevel::Medium
        } else if perf.margin_loss_rate < -5.0 {
            RiskLevel::Low
        } else {
            RiskLevel::Normal
        }
    }
}

// 实现 ExchangeClient trait
#[async_trait]
impl ExchangeClient for BinanceClient {
    fn get_exchange_name(&self) -> &str {
        "Binance"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign_request(&query);

        // 先尝试统一账户端点
        let url_papi = format!(
            "{}/papi/v1/um/positionRisk?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response_papi = client
            .get(&url_papi)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await;

        // 如果统一账户成功，使用它的结果
        if let Ok(resp) = response_papi {
            if resp.status().is_success() {
                if let Ok(body_text) = resp.text().await {
                    let full_response: String = body_text.chars().take(5000).collect();
                    info!("🔍 PAPI positionRisk 完整响应: {}", full_response);

                    // 尝试解析数组格式
                    if let Ok(positions) = serde_json::from_str::<Vec<PositionRisk>>(&body_text) {
                        let active_positions: Vec<Position> = positions
                            .into_iter()
                            .filter(|p| p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
                            .map(|p| {
                                let amt = p.positionAmt.parse::<f64>().unwrap_or(0.0);
                                Position {
                                    symbol: p.symbol,
                                    side: if amt > 0.0 {
                                        "LONG".to_string()
                                    } else {
                                        "SHORT".to_string()
                                    },
                                    size: amt.abs(),
                                    entry_price: p.entryPrice.parse().unwrap_or(0.0),
                                    mark_price: p.markPrice.parse().unwrap_or(0.0),
                                    pnl: p.unRealizedProfit.parse().unwrap_or(0.0),
                                    leverage: p.leverage.parse().unwrap_or(1),
                                    margin: 0.0,
                                }
                            })
                            .collect();
                        info!(
                            "✅ PAPI持仓查询成功(数组格式): {} 个持仓",
                            active_positions.len()
                        );
                        return Ok(active_positions);
                    }

                    // 尝试解析 map 格式 { symbol: {...} }
                    if let Ok(positions_map) =
                        serde_json::from_str::<HashMap<String, PositionRisk>>(&body_text)
                    {
                        let active_positions: Vec<Position> = positions_map
                            .into_iter()
                            .filter(|(_, p)| {
                                p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0
                            })
                            .map(|(_, p)| {
                                let amt = p.positionAmt.parse::<f64>().unwrap_or(0.0);
                                Position {
                                    symbol: p.symbol,
                                    side: if amt > 0.0 {
                                        "LONG".to_string()
                                    } else {
                                        "SHORT".to_string()
                                    },
                                    size: amt.abs(),
                                    entry_price: p.entryPrice.parse().unwrap_or(0.0),
                                    mark_price: p.markPrice.parse().unwrap_or(0.0),
                                    pnl: p.unRealizedProfit.parse().unwrap_or(0.0),
                                    leverage: p.leverage.parse().unwrap_or(1),
                                    margin: 0.0,
                                }
                            })
                            .collect();
                        info!(
                            "✅ PAPI持仓查询成功(map格式): {} 个持仓",
                            active_positions.len()
                        );
                        return Ok(active_positions);
                    }

                    #[derive(Deserialize)]
                    struct WrappedResponse {
                        data: serde_json::Value,
                    }

                    if let Ok(wrapped) = serde_json::from_str::<WrappedResponse>(&body_text) {
                        if let Ok(positions) =
                            serde_json::from_value::<Vec<PositionRisk>>(wrapped.data.clone())
                        {
                            let active_positions: Vec<Position> = positions
                                .into_iter()
                                .filter(|p| p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
                                .map(|p| {
                                    let amt = p.positionAmt.parse::<f64>().unwrap_or(0.0);
                                    Position {
                                        symbol: p.symbol,
                                        side: if amt > 0.0 {
                                            "LONG".to_string()
                                        } else {
                                            "SHORT".to_string()
                                        },
                                        size: amt.abs(),
                                        entry_price: p.entryPrice.parse().unwrap_or(0.0),
                                        mark_price: p.markPrice.parse().unwrap_or(0.0),
                                        pnl: p.unRealizedProfit.parse().unwrap_or(0.0),
                                        leverage: p.leverage.parse().unwrap_or(1),
                                        margin: 0.0,
                                    }
                                })
                                .collect();
                            info!(
                                "✅ PAPI持仓查询成功(包装数组): {} 个持仓",
                                active_positions.len()
                            );
                            return Ok(active_positions);
                        }

                        if let Ok(positions_map) =
                            serde_json::from_value::<HashMap<String, PositionRisk>>(wrapped.data)
                        {
                            let active_positions: Vec<Position> = positions_map
                                .into_iter()
                                .filter(|(_, p)| {
                                    p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0
                                })
                                .map(|(_, p)| {
                                    let amt = p.positionAmt.parse::<f64>().unwrap_or(0.0);
                                    Position {
                                        symbol: p.symbol,
                                        side: if amt > 0.0 {
                                            "LONG".to_string()
                                        } else {
                                            "SHORT".to_string()
                                        },
                                        size: amt.abs(),
                                        entry_price: p.entryPrice.parse().unwrap_or(0.0),
                                        mark_price: p.markPrice.parse().unwrap_or(0.0),
                                        pnl: p.unRealizedProfit.parse().unwrap_or(0.0),
                                        leverage: p.leverage.parse().unwrap_or(1),
                                        margin: 0.0,
                                    }
                                })
                                .collect();
                            info!(
                                "✅ PAPI持仓查询成功(包装map): {} 个持仓",
                                active_positions.len()
                            );
                            return Ok(active_positions);
                        }
                    }

                    error!("❌ PAPI持仓数据格式无法识别,回退到FAPI");
                    error!("响应前500字符: {}", &body_text[..body_text.len().min(500)]);
                }
            }
        }

        // 回退到普通合约端点
        let url_fapi = format!(
            "{}/fapi/v2/positionRisk?{}&signature={}",
            self.base_url, query, signature
        );

        // 重新创建client因为前面的请求已经消费了
        let client = self.create_ipv4_client()?;
        let positions: Vec<PositionRisk> = client
            .get(&url_fapi)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?
            .json()
            .await?;

        let active_positions: Vec<Position> = positions
            .into_iter()
            .filter(|p| p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
            .map(|p| {
                let amt = p.positionAmt.parse::<f64>().unwrap_or(0.0);
                Position {
                    symbol: p.symbol,
                    side: if amt > 0.0 {
                        "LONG".to_string()
                    } else {
                        "SHORT".to_string()
                    },
                    size: amt.abs(),
                    entry_price: p.entryPrice.parse().unwrap_or(0.0),
                    mark_price: p.markPrice.parse().unwrap_or(0.0),
                    pnl: p.unRealizedProfit.parse().unwrap_or(0.0),
                    leverage: p.leverage.parse().unwrap_or(1),
                    margin: 0.0, // Binance API 不直接提供，需要计算
                }
            })
            .collect();

        Ok(active_positions)
    }

    async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign_request(&query);

        let client = self.create_ipv4_client()?;
        let mut total = 0.0;
        let mut available = 0.0;
        let mut pnl = 0.0;

        // 1. 尝试统一账户端点 (papi) - 包含合约、现货等
        let url_papi = format!(
            "{}/papi/v1/balance?{}&signature={}",
            self.papi_base_url, query, signature
        );

        let response = client
            .get(&url_papi)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;

            #[derive(Debug, Deserialize)]
            #[allow(non_snake_case)]
            struct UnifiedAsset {
                asset: String,
                totalWalletBalance: String,
                umWalletBalance: String,
                cmWalletBalance: String,
                crossMarginAsset: String,
                umUnrealizedPNL: String,
            }

            if let Ok(assets) = serde_json::from_str::<Vec<UnifiedAsset>>(&body) {
                for asset in assets {
                    let total_wallet = asset.totalWalletBalance.parse::<f64>().unwrap_or(0.0);
                    let um_balance = asset.umWalletBalance.parse::<f64>().unwrap_or(0.0);
                    let cm_balance = asset.cmWalletBalance.parse::<f64>().unwrap_or(0.0);
                    let cross_margin = asset.crossMarginAsset.parse::<f64>().unwrap_or(0.0);

                    if asset.asset == "USDT" || asset.asset == "USDC" {
                        total += total_wallet;
                        available += um_balance;
                        pnl += asset.umUnrealizedPNL.parse::<f64>().unwrap_or(0.0);

                        if um_balance > 0.01 {
                            info!("Binance U本位合约: {:.2} {}", um_balance, asset.asset);
                        }
                        if cm_balance > 0.01 {
                            info!("Binance 币本位合约: {:.2} {}", cm_balance, asset.asset);
                        }
                        if cross_margin > 0.01 {
                            info!("Binance 杠杆账户: {:.2} {}", cross_margin, asset.asset);
                        }
                    }
                }

                // 2. 查询现货账户
                let spot_query = format!("timestamp={}", chrono::Utc::now().timestamp_millis());
                let spot_sig = self.sign_request(&spot_query);
                let url_spot = format!(
                    "https://api.binance.com/api/v3/account?{}&signature={}",
                    spot_query, spot_sig
                );

                info!("查询 Binance 现货账户...");
                if let Ok(spot_resp) = client
                    .get(&url_spot)
                    .header("X-MBX-APIKEY", &self.api_key)
                    .send()
                    .await
                {
                    if spot_resp.status().is_success() {
                        if let Ok(spot_body) = spot_resp.text().await {
                            #[derive(Debug, Deserialize)]
                            struct SpotBalance {
                                asset: String,
                                free: String,
                                locked: String,
                            }
                            #[derive(Debug, Deserialize)]
                            struct SpotAccount {
                                balances: Vec<SpotBalance>,
                            }

                            if let Ok(spot_account) =
                                serde_json::from_str::<SpotAccount>(&spot_body)
                            {
                                for balance in spot_account.balances {
                                    if balance.asset == "USDT" || balance.asset == "USDC" {
                                        let free = balance.free.parse::<f64>().unwrap_or(0.0);
                                        let locked = balance.locked.parse::<f64>().unwrap_or(0.0);
                                        let spot_total = free + locked;

                                        if spot_total > 0.0001 {
                                            info!(
                                                "Binance 现货账户 {}: {:.2}",
                                                balance.asset, spot_total
                                            );
                                            total += spot_total;
                                            available += free;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 3. 查询资金账户 (使用官方资金钱包API)
                // POST /sapi/v1/asset/get-funding-asset
                let fund_timestamp = chrono::Utc::now().timestamp_millis();
                let fund_query = format!("timestamp={}", fund_timestamp);
                let fund_sig = self.sign_request(&fund_query);
                let url_fund = format!(
                    "https://api.binance.com/sapi/v1/asset/get-funding-asset?{}&signature={}",
                    fund_query, fund_sig
                );

                info!("🔍 查询 Binance 资金账户（Funding Wallet）...");
                if let Ok(fund_resp) = client
                    .post(&url_fund)
                    .header("X-MBX-APIKEY", &self.api_key)
                    .send()
                    .await
                {
                    let status = fund_resp.status();
                    if status.is_success() {
                        if let Ok(fund_body) = fund_resp.text().await {
                            #[derive(Debug, Deserialize)]
                            #[allow(non_snake_case)]
                            struct FundingAsset {
                                asset: String,
                                free: String,
                                locked: String,
                                freeze: String,
                                withdrawing: String,
                                btcValuation: String,
                            }

                            if let Ok(funding_assets) =
                                serde_json::from_str::<Vec<FundingAsset>>(&fund_body)
                            {
                                for asset in funding_assets {
                                    let free = asset.free.parse::<f64>().unwrap_or(0.0);
                                    let locked = asset.locked.parse::<f64>().unwrap_or(0.0);
                                    let freeze = asset.freeze.parse::<f64>().unwrap_or(0.0);
                                    let fund_total = free + locked + freeze;

                                    if fund_total > 0.00001 {
                                        // 统计 USDT 和 USDC
                                        if asset.asset == "USDT" || asset.asset == "USDC" {
                                            info!(
                                                "Binance 资金账户 {}: {:.2}",
                                                asset.asset, fund_total
                                            );
                                            total += fund_total;
                                            available += free;
                                        }
                                    }
                                }
                            } else {
                                warn!("❌ 解析资金账户响应失败");
                            }
                        }
                    } else {
                        warn!("⚠️ 资金账户 API 返回错误: {} ({})", status, status.as_u16());
                        if let Ok(error_body) = fund_resp.text().await {
                            warn!("错误详情: {}", &error_body[..error_body.len().min(200)]);
                        }
                    }
                } else {
                    warn!("⚠️ 资金账户 API 请求失败");
                }

                return Ok(AccountInfo {
                    total_balance: total,
                    available_balance: available,
                    unrealized_pnl: pnl,
                    margin_used: total - available,
                });
            }
        }

        // 如果统一账户失败，尝试普通合约端点 (fapi)
        let url_fapi = format!(
            "{}/fapi/v2/account?{}&signature={}",
            self.base_url, query, signature
        );

        let response_fapi = client
            .get(&url_fapi)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        let status_fapi = response_fapi.status();
        let body_fapi = response_fapi.text().await?;

        if !status_fapi.is_success() {
            error!("Binance 获取账户信息失败: {}", body_fapi);
            return Err(anyhow::anyhow!("Binance API错误: {}", body_fapi));
        }

        let account: AccountInformation = serde_json::from_str(&body_fapi)?;

        Ok(AccountInfo {
            total_balance: account.totalWalletBalance.parse().unwrap_or(0.0),
            available_balance: account.availableBalance.parse().unwrap_or(0.0),
            unrealized_pnl: account.totalUnrealizedProfit.parse().unwrap_or(0.0),
            margin_used: 0.0,
        })
    }

    async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        // 直接实现以避免递归调用
        let url = format!("{}/fapi/v1/ticker/price?symbol={}", self.base_url, symbol);

        let client = self.create_ipv4_client()?;
        let response: serde_json::Value = client.get(&url).send().await?.json().await?;

        let price: f64 = response["price"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("价格解析失败"))?
            .parse()?;

        Ok(price)
    }

    async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        // 从缓存或 API 获取交易规则
        {
            let cache = self.rules_cache.read().await;
            if let Some(rules) = cache.get(symbol) {
                return Ok(rules.clone());
            }
        }

        // 调用原有的方法获取规则
        BinanceClient::get_symbol_trading_rules(self, symbol).await
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        self.change_leverage(symbol, leverage).await
    }

    async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()> {
        self.set_margin_type(symbol, margin_type)
            .await
            .or_else(|e| {
                let err_str = e.to_string().to_lowercase();
                // 统一账户(Portfolio Margin)强制全仓,调用set_margin_type会返回-2015权限错误
                // 标准合约账户如果已是目标模式会返回"no need to change"
                // 两种情况都应该忽略错误继续交易
                if err_str.contains("no need to change")
                    || err_str.contains("-2015")
                    || err_str.contains("invalid api-key")
                {
                    warn!("⚠️  设置margin_type被跳过 (可能是统一账户): {}", err_str);
                    Ok(())
                } else {
                    Err(e)
                }
            })
    }

    async fn set_position_mode(&self, dual_side: bool) -> Result<()> {
        self.set_position_mode(dual_side).await.or_else(|e| {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("no need to change") || err_str.contains("not modified") {
                Ok(())
            } else {
                warn!("Binance设置持仓模式警告: {}", e);
                Ok(()) // 不阻塞交易
            }
        })
    }

    async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side: bool,
    ) -> Result<OrderResult> {
        let _ = self.set_position_mode(dual_side).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 1.001; // 限价稍高以提高成交概率

        let order_id = self
            .limit_order(symbol, quantity, "BUY", limit_price, Some("LONG"))
            .await?;

        info!(
            "✅ Binance开多限价单已提交: {} 数量: {} 价格: {}",
            symbol, quantity, limit_price
        );
        Ok(OrderResult {
            order_id,
            symbol: symbol.to_string(),
            side: "BUY".to_string(),
            quantity,
            price: limit_price,
            status: "FILLED".to_string(),
        })
    }

    async fn open_short(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side: bool,
    ) -> Result<OrderResult> {
        let _ = self.set_position_mode(dual_side).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 0.999; // 限价稍低以提高成交概率

        let order_id = self
            .limit_order(symbol, quantity, "SELL", limit_price, Some("SHORT"))
            .await?;

        info!(
            "✅ Binance开空限价单已提交: {} 数量: {} 价格: {}",
            symbol, quantity, limit_price
        );
        Ok(OrderResult {
            order_id,
            symbol: symbol.to_string(),
            side: "SELL".to_string(),
            quantity,
            price: limit_price,
            status: "FILLED".to_string(),
        })
    }

    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };
        self.market_order(symbol, size, close_side).await?;

        info!("✅ Binance平仓成功: {} {} {}", symbol, side, size);
        Ok(OrderResult {
            order_id: "".to_string(),
            symbol: symbol.to_string(),
            side: close_side.to_string(),
            quantity: size,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Vec<f64>>> {
        let limit_value = limit.unwrap_or(100);
        let url = format!(
            "{}/fapi/v1/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit_value
        );

        let client = self.create_ipv4_client()?;
        let response_text = client.get(&url).send().await?.text().await?;

        let klines_raw: Vec<serde_json::Value> =
            if let Ok(array) = serde_json::from_str::<Vec<serde_json::Value>>(&response_text) {
                array
            } else if let Ok(map) =
                serde_json::from_str::<HashMap<String, serde_json::Value>>(&response_text)
            {
                map.into_values()
                    .next()
                    .and_then(|value| value.as_array().cloned())
                    .ok_or_else(|| anyhow::anyhow!("K线数据格式错误: map中无有效数组"))?
            } else {
                let preview: String = response_text.chars().take(200).collect();
                return Err(anyhow::anyhow!("无法解析K线响应: {}", preview));
            };

        let klines: Vec<Vec<f64>> = klines_raw
            .iter()
            .map(|k| {
                vec![
                    k[0].as_i64().unwrap_or(0) as f64,                   // timestamp
                    k[1].as_str().unwrap_or("0").parse().unwrap_or(0.0), // open
                    k[2].as_str().unwrap_or("0").parse().unwrap_or(0.0), // high
                    k[3].as_str().unwrap_or("0").parse().unwrap_or(0.0), // low
                    k[4].as_str().unwrap_or("0").parse().unwrap_or(0.0), // close
                    k[5].as_str().unwrap_or("0").parse().unwrap_or(0.0), // volume
                ]
            })
            .collect();

        Ok(klines)
    }

    async fn adjust_position(
        &self,
        symbol: &str,
        side: &str,
        quantity_delta: f64,
        leverage: u32,
        margin_type: &str,
    ) -> Result<OrderResult> {
        if quantity_delta.abs() < f64::EPSILON {
            return Ok(OrderResult {
                order_id: String::new(),
                symbol: symbol.to_string(),
                side: side.to_string(),
                quantity: 0.0,
                price: 0.0,
                status: "SKIPPED".to_string(),
            });
        }

        if quantity_delta > 0.0 {
            // 加仓
            if side.eq_ignore_ascii_case("LONG") {
                <Self as ExchangeClient>::open_long(
                    self,
                    symbol,
                    quantity_delta,
                    leverage,
                    margin_type,
                    false,
                )
                .await
            } else {
                <Self as ExchangeClient>::open_short(
                    self,
                    symbol,
                    quantity_delta,
                    leverage,
                    margin_type,
                    false,
                )
                .await
            }
        } else {
            // 减仓
            let reduce_amount = quantity_delta.abs();
            <Self as ExchangeClient>::close_position(self, symbol, side, reduce_amount).await
        }
    }
}
