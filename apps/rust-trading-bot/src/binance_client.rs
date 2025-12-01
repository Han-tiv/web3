use crate::exchange_trait::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use log::{debug, error, info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct AccountInformation {
    pub totalWalletBalance: String,
    pub totalMarginBalance: String,
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

/// 单个订单的精简状态信息，方便上层策略快速查看成交进度
#[derive(Debug, Clone)]
pub struct OrderStatus {
    pub order_id: String,
    pub status: String,
    pub executed_qty: f64,
    pub orig_qty: f64,
    pub price: f64,
    pub stop_price: Option<f64>,
}

/// Binance 未完成订单的精简视图
#[derive(Debug, Clone, Serialize)]
pub struct OpenOrder {
    pub order_id: String,
    pub symbol: String,
    pub order_type: String,
    pub status: String,
    pub reduce_only: bool,
    pub created_at: DateTime<Utc>,
    pub side: Option<String>,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub quantity: Option<f64>,
    pub position_side: Option<String>,
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
    notional: Option<String>,
    minNotional: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct RawOpenOrder {
    orderId: i64,
    symbol: String,
    status: Option<String>,
    r#type: String,
    side: Option<String>,
    reduceOnly: Option<bool>,
    price: Option<String>,
    stopPrice: Option<String>,
    origQty: Option<String>,
    positionSide: Option<String>,
    updateTime: Option<i64>,
    time: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct BinanceClient {
    api_key: String,
    secret_key: String,
    base_url: String, // FAPI endpoint (fapi.binance.com)
    // 缓存每个交易对的交易规则，减少 exchangeInfo 请求
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
    // 缓存账户持仓模式: true=双向, false=单向
    position_mode_cache: Arc<RwLock<Option<bool>>>,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            // Testnet uses unified domain for futures
            "https://testnet.binancefuture.com".to_string()
        } else {
            // Mainnet futures endpoint
            "https://fapi.binance.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
            position_mode_cache: Arc::new(RwLock::new(None)),
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

    /// 获取指定交易对或全量的未完成委托
    pub async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<OpenOrder>> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let mut query = format!("timestamp={}", timestamp);
        if let Some(sym) = symbol {
            query = format!("symbol={}&{}", sym, query);
        }

        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/openOrders?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let body = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?
            .text()
            .await?;

        let raw_orders: Vec<RawOpenOrder> = serde_json::from_str(&body).map_err(|err| {
            let preview: String = body.chars().take(400).collect();
            error!("❌ 解析未完成订单失败: {} | 响应片段: {}", err, preview);
            anyhow::anyhow!("解析未完成订单失败: {}", err)
        })?;

        let parse_number =
            |value: Option<String>| -> Option<f64> { value.and_then(|v| v.parse::<f64>().ok()) };

        let orders = raw_orders
            .into_iter()
            .map(|raw| {
                let ts = raw.updateTime.or(raw.time).unwrap_or(timestamp);
                let created_at = Utc
                    .timestamp_millis_opt(ts)
                    .single()
                    .unwrap_or_else(|| Utc::now());

                OpenOrder {
                    order_id: raw.orderId.to_string(),
                    symbol: raw.symbol,
                    order_type: raw.r#type,
                    status: raw.status.unwrap_or_else(|| "UNKNOWN".to_string()),
                    reduce_only: raw.reduceOnly.unwrap_or(false),
                    created_at,
                    side: raw.side,
                    price: parse_number(raw.price),
                    stop_price: parse_number(raw.stopPrice),
                    quantity: parse_number(raw.origQty),
                    position_side: raw.positionSide,
                }
            })
            .collect();

        Ok(orders)
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
        info!("合约余额: {} USDT", account.totalMarginBalance);
        info!("未实现盈亏: {} USDT", account.totalUnrealizedProfit);
        Ok(account)
    }

    pub async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        _dual_side_position: bool,
    ) -> Result<()> {
        // 强制设置为单向持仓模式
        let _ = self.set_position_mode(false).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        // 使用当前价格略微加价，提升限价单成交概率
        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 1.001;

        let _order_id = self
            .limit_order(
                symbol,
                quantity,
                "BUY",
                limit_price,
                None, // 单向持仓不需要positionSide
                false,
            )
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
        _dual_side_position: bool,
    ) -> Result<()> {
        // 强制设置为单向持仓模式
        let _ = self.set_position_mode(false).await;
        let _ = self.set_margin_type(symbol, margin_type).await;
        self.change_leverage(symbol, leverage).await?;

        // 使用当前价格略微减价，提升限价单成交概率
        let current_price = self.get_current_price(symbol).await?;
        let limit_price = current_price * 0.999;

        let _order_id = self
            .limit_order(
                symbol,
                quantity,
                "SELL",
                limit_price,
                None, // 单向持仓不需要positionSide
                false,
            )
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
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        // 先保存status,因为response.text()会消费response
        let status = response.status();

        // 检查HTTP状态码
        if !status.is_success() {
            let body = response.text().await?;
            error!("❌ 设置杠杆失败: HTTP {} | {}", status, body);
            return Err(anyhow::anyhow!("设置杠杆失败: {}", body));
        }

        // 解析响应JSON并验证实际设置的杠杆值
        let body = response.text().await?;
        let result: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("解析杠杆响应失败: {} | 响应: {}", e, body))?;

        // Binance API 返回格式: {"leverage": 20, "maxNotionalValue": "...", "symbol": "BTCUSDT"}
        let actual_leverage = result["leverage"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法解析杠杆返回值 | 响应: {}", body))?
            as u32;

        // 验证实际杠杆与请求杠杆是否一致
        if actual_leverage != leverage {
            warn!(
                "⚠️  {} 杠杆设置与预期不符! 请求: {}x → 实际: {}x (可能被Binance后台限制)",
                symbol, leverage, actual_leverage
            );
            warn!("   建议: 1. 登录Binance检查账户杠杆限制  2. 修改代码配置以匹配实际杠杆");
        } else {
            info!("✅ {} 杠杆已成功设置为 {}x", symbol, actual_leverage);
        }

        Ok(())
    }

    pub async fn market_order(&self, symbol: &str, quantity: f64, side: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 获取交易规则与当前价格
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let current_price = self.get_current_price(symbol).await?;

        let min_notional = rules.min_notional.unwrap_or(5.0);

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

        let query = format!(
            "symbol={}&side={}&type=MARKET&quantity={}&timestamp={}",
            symbol, side, quantity_str, timestamp
        );
        let signature = self.sign_request(&query);

        // 统一使用经典合约(FAPI)下单
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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

    /// 获取资金费率信息
    /// 返回: (当前资金费率, 下次费率时间戳, 标记价格, 现货价格, 溢价率)
    pub async fn get_funding_rate(&self, symbol: &str) -> Result<(f64, i64, f64, f64, f64)> {
        let url = format!("{}/fapi/v1/premiumIndex?symbol={}", self.base_url, symbol);

        let client = self.create_ipv4_client()?;
        let response: serde_json::Value = client.get(&url).send().await?.json().await?;

        let funding_rate: f64 = response["lastFundingRate"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("资金费率解析失败"))?
            .parse()?;

        let next_funding_time: i64 = response["nextFundingTime"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("下次资金费率时间解析失败"))?;

        let mark_price: f64 = response["markPrice"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        let index_price: f64 = response["indexPrice"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        // 计算溢价率 (mark_price - index_price) / index_price
        let premium_rate = if index_price > 0.0 {
            ((mark_price - index_price) / index_price) * 100.0
        } else {
            0.0
        };

        Ok((
            funding_rate,
            next_funding_time,
            mark_price,
            index_price,
            premium_rate,
        ))
    }

    /// 获取历史资金费率
    /// limit: 返回最近N条记录 (默认100, 最大1000)
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(i64, f64)>> {
        let limit_value = limit.unwrap_or(100).min(1000);
        let url = format!(
            "{}/fapi/v1/fundingRate?symbol={}&limit={}",
            self.base_url, symbol, limit_value
        );

        let client = self.create_ipv4_client()?;
        let response: Vec<serde_json::Value> = client.get(&url).send().await?.json().await?;

        let history: Vec<(i64, f64)> = response
            .iter()
            .filter_map(|record| {
                let timestamp = record["fundingTime"].as_i64()?;
                let rate = record["fundingRate"].as_str()?.parse::<f64>().ok()?;
                Some((timestamp, rate))
            })
            .collect();

        Ok(history)
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
                let mut min_notional_val = None;

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
                    if filter.filterType == "MIN_NOTIONAL" {
                        if let Some(value) = filter
                            .notional
                            .as_ref()
                            .or_else(|| filter.minNotional.as_ref())
                        {
                            min_notional_val = value.parse::<f64>().ok();
                        }
                    }
                }

                let rules = TradingRules {
                    step_size: step_size_val.ok_or_else(|| anyhow::anyhow!("缺少stepSize信息"))?,
                    min_qty: min_qty_val.ok_or_else(|| anyhow::anyhow!("缺少minQty信息"))?,
                    quantity_precision: symbol_info.quantityPrecision,
                    price_precision: symbol_info.pricePrecision,
                    tick_size: tick_size_val.ok_or_else(|| anyhow::anyhow!("缺少tickSize信息"))?,
                    min_notional: min_notional_val,
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
            if body_lower.contains("no need to change") || body_lower.contains("not modified") {
                warn!("⚠️  设置持仓模式被跳过 (可能是统一账户): {}", body);
                let mut cache = self.position_mode_cache.write().await;
                *cache = Some(dual_side_position);
                return Ok(());
            }
            if body_lower.contains("-2015") || body_lower.contains("invalid api-key") {
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
        {
            let mut cache = self.position_mode_cache.write().await;
            *cache = Some(dual_side_position);
        }
        Ok(())
    }

    pub async fn get_position_mode(&self) -> Result<bool> {
        {
            let cache = self.position_mode_cache.read().await;
            if let Some(mode) = *cache {
                return Ok(mode);
            }
        }

        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/positionSide/dual?{}&signature={}",
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
            error!("查询持仓模式失败: {}", body);
            return Err(anyhow::anyhow!("查询持仓模式失败: {}", body));
        }

        let dual_side = serde_json::from_str::<serde_json::Value>(&body)?["dualSidePosition"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("无法解析持仓模式"))?;

        {
            let mut cache = self.position_mode_cache.write().await;
            *cache = Some(dual_side);
        }

        Ok(dual_side)
    }

    /// 设置止损单 (STOP 限价触发)
    pub async fn set_stop_loss(
        &self,
        symbol: &str,
        side: &str, // "LONG" or "SHORT" - 持仓方向
        quantity: f64,
        stop_price: f64,
        limit_price: Option<f64>,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 平多仓用SELL,平空仓用BUY
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };
        // 获取交易规则以便获取精度信息
        let rules = self.get_symbol_trading_rules(symbol).await?;

        // 获取价格精度
        let price_precision = rules.price_precision.max(0) as usize;
        let qty_precision = rules.quantity_precision.max(0) as usize;

        // 获取最新价格,用于止损价格合理性校验
        let current_price = self.get_current_price(symbol).await?;
        let tick_size = rules.tick_size;

        // 按平仓方向选择合适的取整方式,避免止损价偏离预期
        let mut aligned_stop_price = if order_side == "SELL" {
            (stop_price / tick_size).ceil() * tick_size
        } else {
            (stop_price / tick_size).floor() * tick_size
        };

        info!(
            "📐 {} {}止损价格对齐: 原始=${:.8}, tick_size=${:.8}, 对齐后=${:.8}",
            symbol, order_side, stop_price, tick_size, aligned_stop_price
        );

        // 基于最新价格验证止损是否仍在合理区间
        if order_side == "SELL" {
            if aligned_stop_price >= current_price {
                warn!(
                    "⚠️ 多头止损价 {:.8} >= 当前价 {:.8}, 调整为当前价*0.99",
                    aligned_stop_price, current_price
                );
                aligned_stop_price = (current_price * 0.99 / tick_size).floor() * tick_size;
            }
        } else if aligned_stop_price <= current_price {
            warn!(
                "⚠️ 空头止损价 {:.8} <= 当前价 {:.8}, 调整为当前价*1.01",
                aligned_stop_price, current_price
            );
            aligned_stop_price = (current_price * 1.01 / tick_size).ceil() * tick_size;
        }

        let stop_price_str = format!("{:.*}", price_precision, aligned_stop_price);

        // 按 tick_size 对齐限价单价格
        let actual_limit_price = limit_price.unwrap_or(aligned_stop_price);
        let aligned_limit_price = (actual_limit_price / tick_size).round() * tick_size;
        let limit_price_str = format!("{:.*}", price_precision, aligned_limit_price);

        let quantity_str = format!("{:.*}", qty_precision, quantity);

        let query = format!(
            "symbol={}&side={}&type=STOP&stopPrice={}&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, order_side, stop_price_str, limit_price_str, quantity_str, timestamp
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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
            error!("设置止损单失败: {}", body);
            return Err(anyhow::anyhow!("设置止损单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 止损单已设置: {} {} stop=${} limit=${} (订单ID: {})",
            symbol, order_side, stop_price, actual_limit_price, order_id
        );
        Ok(order_id)
    }

    /// 设置止盈单 (TAKE_PROFIT 限价触发)
    pub async fn set_take_profit(
        &self,
        symbol: &str,
        side: &str, // "LONG" or "SHORT" - 持仓方向
        quantity: f64,
        stop_price: f64,
        limit_price: Option<f64>,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 平多仓用SELL,平空仓用BUY
        let order_side = if side == "LONG" { "SELL" } else { "BUY" };

        // 获取交易规则并调整数量
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;
        let quantity_str = format!("{:.*}", qty_precision, quantity);

        // 按 tick_size 对齐止盈触发价,避免 -4014 错误
        let aligned_stop_price = (stop_price / rules.tick_size).round() * rules.tick_size;
        let stop_price_str = format!("{:.*}", price_precision, aligned_stop_price);

        // 按 tick_size 对齐限价单价格
        let actual_limit_price = limit_price.unwrap_or(aligned_stop_price);
        let aligned_limit_price = (actual_limit_price / rules.tick_size).round() * rules.tick_size;
        let limit_price_str = format!("{:.*}", price_precision, aligned_limit_price);

        let query = format!(
            "symbol={}&side={}&type=TAKE_PROFIT&stopPrice={}&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, order_side, stop_price_str, limit_price_str, quantity_str, timestamp
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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
            error!("设置止盈单失败: {}", body);
            return Err(anyhow::anyhow!("设置止盈单失败: {}", body));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "✅ 止盈单已设置: {} {} stop=${} limit=${} (订单ID: {})",
            symbol, order_side, stop_price, actual_limit_price, order_id
        );
        Ok(order_id)
    }

    /// 下触发单 (开仓或平仓的条件单)
    ///
    /// trigger_type: "STOP" | "STOP_MARKET" | "TAKE_PROFIT" | "TAKE_PROFIT_MARKET"
    /// action: "OPEN" (开仓) or "CLOSE" (平仓)
    /// position_side: "LONG" or "SHORT"
    pub async fn place_trigger_order(
        &self,
        symbol: &str,
        trigger_type: &str,  // STOP_MARKET, TAKE_PROFIT_MARKET, 等
        action: &str,        // OPEN / CLOSE
        position_side: &str, // LONG / SHORT
        quantity: f64,
        stop_price: f64,
        limit_price: Option<f64>, // STOP/TAKE_PROFIT 需要的挂单价
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        let trigger_type_upper = trigger_type.trim().to_ascii_uppercase();
        let action_upper = action.trim().to_ascii_uppercase();
        let position_side_upper = position_side.trim().to_ascii_uppercase();

        let valid_trigger_types = ["STOP", "STOP_MARKET", "TAKE_PROFIT", "TAKE_PROFIT_MARKET"];
        if !valid_trigger_types.contains(&trigger_type_upper.as_str()) {
            return Err(anyhow::anyhow!(format!(
                "不支持的触发单类型: {}",
                trigger_type
            )));
        }
        if action_upper != "OPEN" && action_upper != "CLOSE" {
            return Err(anyhow::anyhow!(format!(
                "action 只能是 OPEN/CLOSE, 当前为 {}",
                action
            )));
        }
        if position_side_upper != "LONG" && position_side_upper != "SHORT" {
            return Err(anyhow::anyhow!(format!(
                "position_side 只能是 LONG/SHORT, 当前为 {}",
                position_side
            )));
        }

        // 根据动作与仓位方向确认 Binance 下单 side
        let order_side = match (action_upper.as_str(), position_side_upper.as_str()) {
            ("OPEN", "LONG") => "BUY",
            ("OPEN", "SHORT") => "SELL",
            ("CLOSE", "LONG") => "SELL",
            ("CLOSE", "SHORT") => "BUY",
            _ => unreachable!("已在上方校验 action 与 position_side"),
        };

        // 获取交易规则用于数量与价格精度
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;
        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let stop_price_str = format!("{:.*}", price_precision, stop_price);

        let requires_limit_price = matches!(trigger_type_upper.as_str(), "STOP" | "TAKE_PROFIT");
        let mut query = format!(
            "symbol={}&side={}&type={}&stopPrice={}&quantity={}&workingType=MARK_PRICE&timestamp={}",
            symbol, order_side, trigger_type_upper, stop_price_str, quantity_str, timestamp
        );

        if requires_limit_price {
            let limit = limit_price
                .ok_or_else(|| anyhow::anyhow!("STOP/TAKE_PROFIT 类型必须提供 limit_price"))?;
            let limit_price_str = format!("{:.*}", price_precision, limit);
            query = format!("{}&price={}", query, limit_price_str);
        } else if let Some(limit) = limit_price {
            // 非 STOP/TAKE_PROFIT 传入了限价，按 Binance 要求忽略，仅提示日志便于排查
            debug!(
                "触发单类型 {} 不需要 limit_price, 已忽略传入值 {}",
                trigger_type_upper, limit
            );
        }

        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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
            error!("触发单下单失败: {}", body);
            return Err(anyhow::anyhow!(format!("触发单下单失败: {}", body)));
        }

        let result: serde_json::Value = response.json().await?;
        let order_id = result["orderId"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("无法获取订单ID"))?
            .to_string();

        info!(
            "🎯 触发单已下: {} {} {} @ trigger={} (订单ID: {})",
            symbol, action_upper, position_side_upper, stop_price, order_id
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

        // 获取交易规则
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        // 格式化数量和价格
        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let price_str = format!("{:.*}", price_precision, limit_price);

        let query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, order_side, price_str, quantity_str, timestamp
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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

    /// 通用限价单 (单向持仓模式仅依赖 BUY/SELL side)
    pub async fn limit_order(
        &self,
        symbol: &str,
        quantity: f64,
        side: &str, // "BUY" or "SELL"
        limit_price: f64,
        _position_side: Option<&str>,
        reduce_only: bool,
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        let rules = self.get_symbol_trading_rules(symbol).await?;

        // 先按 tick_size 对齐价格，避免提交非法价格
        let aligned_price = (limit_price / rules.tick_size).floor() * rules.tick_size;
        // 按 step_size 与最小数量对齐，避免买卖量不合规
        let step = rules.step_size;
        let adjusted_quantity = (quantity / step).floor() * step;
        let mut final_quantity = if reduce_only {
            // ✅ reduceOnly 保持真实数量,仅对齐 step_size，避免被强制抬升到 min_qty
            adjusted_quantity.max(step)
        } else if adjusted_quantity < rules.min_qty {
            rules.min_qty
        } else {
            adjusted_quantity
        };

        // 若为普通限价单，自动拉升数量以满足 min_notional 限制
        if !reduce_only {
            if let Some(min_notional) = rules.min_notional {
                let current_notional = final_quantity * aligned_price;
                if current_notional < min_notional {
                    let previous_quantity = final_quantity;
                    let required_qty = (min_notional / aligned_price).ceil();
                    // 计算所需数量并按照步长向上对齐
                    final_quantity = ((required_qty / step).ceil()) * step;

                    let new_notional = final_quantity * aligned_price;
                    warn!(
                        "📊 {} 限价单自动提升数量: {:.8} → {:.8} (名义金额 {:.2} → {:.2} USDT)",
                        symbol, previous_quantity, final_quantity, current_notional, new_notional
                    );
                }
            }
        }

        if final_quantity <= 0.0 {
            return Err(anyhow::anyhow!(
                "订单数量过小 ({:.8}),无法下单",
                final_quantity
            ));
        }

        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        let quantity_str = format!("{:.*}", qty_precision, final_quantity);
        let price_str = format!("{:.*}", price_precision, aligned_price);

        let notional_value = final_quantity * aligned_price;
        let min_notional = rules.min_notional.unwrap_or(5.0);

        if !reduce_only && notional_value < min_notional {
            return Err(anyhow::anyhow!(format!(
                "限价单名义金额 {:.4} < 最低要求 {:.2} (数量: {:.6}, 价格: {:.6})",
                notional_value, min_notional, final_quantity, aligned_price
            )));
        } else if reduce_only && notional_value < min_notional {
            warn!(
                "⚠️ {} 减仓金额 {:.4} 低于默认门槛 {:.2}，使用 reduceOnly 放行",
                symbol, notional_value, min_notional
            );
        }

        // 单向持仓模式: 不添加positionSide参数
        // 双向持仓模式: 添加positionSide=LONG/SHORT参数
        let mut query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, side, price_str, quantity_str, timestamp
        );

        // 对于reduce_only订单，添加reduceOnly标记
        if reduce_only {
            query.push_str("&reduceOnly=true");
        }

        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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
        _position_side: Option<&str>, // 单向持仓下忽略 positionSide
    ) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 获取交易规则
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let qty_precision = rules.quantity_precision.max(0) as usize;
        let price_precision = rules.price_precision.max(0) as usize;

        // 格式化数量和价格
        let quantity_str = format!("{:.*}", qty_precision, quantity);
        let price_str = format!("{:.*}", price_precision, limit_price);

        if let Some(min_notional) = rules.min_notional {
            if quantity * limit_price < min_notional {
                return Err(anyhow::anyhow!(format!(
                    "限价单名义金额 {:.4} < 最低要求 {:.2} (数量: {:.6}, 价格: {:.6})",
                    quantity * limit_price,
                    min_notional,
                    quantity,
                    limit_price
                )));
            }
        }

        // 构建查询参数
        let query = format!(
            "symbol={}&side={}&type=LIMIT&price={}&quantity={}&timeInForce=GTC&timestamp={}",
            symbol, side, price_str, quantity_str, timestamp
        );

        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
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

        // 统一使用 FAPI 端点取消
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
            self.base_url, query, signature
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

    /// 查询订单状态详情
    pub async fn get_order_status_detail(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderStatus> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );
        let signature = self.sign_request(&query);

        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let response = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await?;

        if !status_code.is_success() {
            error!("查询订单状态失败: {}", body);
            return Err(anyhow::anyhow!("查询订单状态失败: {}", body));
        }

        let raw: serde_json::Value = serde_json::from_str(&body)?;

        let api_order_id = raw["orderId"]
            .as_i64()
            .map(|id| id.to_string())
            .or_else(|| raw["orderId"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| anyhow::anyhow!("响应缺少 orderId 字段"))?;

        let status_text = raw["status"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("响应缺少 status 字段"))?
            .to_string();

        let executed_qty = if let Some(qty_str) = raw["executedQty"].as_str() {
            qty_str.parse::<f64>()?
        } else if let Some(qty_num) = raw["executedQty"].as_f64() {
            qty_num
        } else {
            return Err(anyhow::anyhow!("响应缺少 executedQty 字段"));
        };

        let orig_qty = if let Some(qty_str) = raw["origQty"].as_str() {
            qty_str.parse::<f64>()?
        } else if let Some(qty_num) = raw["origQty"].as_f64() {
            qty_num
        } else {
            return Err(anyhow::anyhow!("响应缺少 origQty 字段"));
        };

        let price = raw["price"]
            .as_str()
            .and_then(|price_str| price_str.parse::<f64>().ok())
            .or_else(|| raw["price"].as_f64())
            .unwrap_or(0.0);

        let stop_price = raw["stopPrice"]
            .as_str()
            .and_then(|price_str| price_str.parse::<f64>().ok())
            .or_else(|| raw["stopPrice"].as_f64())
            .filter(|value| value.is_finite() && *value > 0.0);

        info!(
            "订单状态: {} (订单ID: {}, 已成交 {} / {})",
            status_text, api_order_id, executed_qty, orig_qty
        );

        Ok(OrderStatus {
            order_id: api_order_id,
            status: status_text,
            executed_qty,
            orig_qty,
            price,
            stop_price,
        })
    }

    /// 查询订单状态文本 (仅返回 status 字段)
    pub async fn get_order_status(&self, symbol: &str, order_id: &str) -> Result<String> {
        let detail = self.get_order_status_detail(symbol, order_id).await?;
        Ok(detail.status)
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
            "{}/fapi/v1/income?{}&signature={}",
            self.base_url, query, signature
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
            "{}/fapi/v1/userTrades?{}&signature={}",
            self.base_url, query, signature
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

        let url = format!(
            "{}/fapi/v2/positionRisk?{}&signature={}",
            self.base_url, query, signature
        );

        let client = self.create_ipv4_client()?;
        let body = client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?
            .text()
            .await?;

        let positions: Vec<PositionRisk> = serde_json::from_str(&body).map_err(|err| {
            let preview: String = body.chars().take(500).collect();
            error!("❌ 解析FAPI持仓失败: {} | 响应前500字符: {}", err, preview);
            anyhow::anyhow!("解析FAPI持仓失败: {}", err)
        })?;

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
        // 仅查询合约账户 (FAPI)，以便反映真实保证金权益
        let futures_account = BinanceClient::get_account_info(self).await?;

        let total = futures_account
            .totalMarginBalance
            .parse::<f64>()
            .unwrap_or(0.0);
        let available = futures_account
            .availableBalance
            .parse::<f64>()
            .unwrap_or(0.0);
        let pnl = futures_account
            .totalUnrealizedProfit
            .parse::<f64>()
            .unwrap_or(0.0);

        Ok(AccountInfo {
            total_balance: total,
            available_balance: available,
            unrealized_pnl: pnl,
            margin_used: total - available,
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
            .limit_order(symbol, quantity, "BUY", limit_price, Some("LONG"), false)
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
            .limit_order(symbol, quantity, "SELL", limit_price, Some("SHORT"), false)
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
                    k[0].as_i64().unwrap_or(0) as f64,                    // timestamp
                    k[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // open
                    k[2].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // high
                    k[3].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // low
                    k[4].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // close
                    k[5].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // volume
                    k[7].as_str().unwrap_or("0").parse().unwrap_or(0.0),  // quote_volume (成交额)
                    k[9].as_str().unwrap_or("0").parse().unwrap_or(0.0), // taker_buy_volume (主动买入量)
                    k[10].as_str().unwrap_or("0").parse().unwrap_or(0.0), // taker_buy_quote_volume (净流入)
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
