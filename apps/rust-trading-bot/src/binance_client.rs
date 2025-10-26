use anyhow::Result;
use hmac::{Hmac, Mac};
use log::{error, info};
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: String,
    pub size: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub pnl: f64,
    pub leverage: i32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct AccountInformation {
    pub totalWalletBalance: String,
    pub availableBalance: String,
    pub totalUnrealizedProfit: String,
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
}

#[derive(Debug, Clone)]
pub struct TradingRules {
    pub step_size: f64,
    pub min_qty: f64,
    pub quantity_precision: i32,
}

#[derive(Debug, Clone)]
pub struct BinanceClient {
    api_key: String,
    secret_key: String,
    base_url: String,
    // 缓存每个交易对的交易规则，减少 exchangeInfo 请求
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

impl BinanceClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            "https://fapi.binance.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
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

        let client = reqwest::Client::new();
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
        info!(
            "未实现盈亏: {} USDT",
            account.totalUnrealizedProfit
        );
        Ok(account)
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign_request(&query);
        let url = format!(
            "{}/fapi/v2/positionRisk?{}&signature={}",
            self.base_url, query, signature
        );

        let client = reqwest::Client::new();
        let positions: Vec<PositionRisk> = client
            .get(&url)
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
                }
            })
            .collect();

        info!("当前持仓数: {}", active_positions.len());
        Ok(active_positions)
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

        self.market_order(symbol, quantity, "BUY").await?;
        info!(
            "✅ 开多成功: {} x{} 杠杆, 数量: {}",
            symbol, leverage, quantity
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

        self.market_order(symbol, quantity, "SELL").await?;
        info!(
            "✅ 开空成功: {} x{} 杠杆, 数量: {}",
            symbol, leverage, quantity
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

        let client = reqwest::Client::new();
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
                info!("ℹ️  仓位模式已为 {}（{}），无需调整", margin_type, symbol);
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

        let client = reqwest::Client::new();
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

        let client = reqwest::Client::new();
        client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        Ok(())
    }

    async fn market_order(&self, symbol: &str, quantity: f64, side: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        // 获取交易规则与当前价格
        let rules = self.get_symbol_trading_rules(symbol).await?;
        let current_price = self.get_current_price(symbol).await?;

        // 名义金额兜底 ≥ 5 USDT
        let mut qty = quantity;
        if qty * current_price < 5.0 {
            qty = 5.0 / current_price;
        }

        // 按 stepSize 向下对齐
        let step = rules.step_size;
        let mut adjusted_quantity = (qty / step).floor() * step;

        // 确保不低于最小数量
        if adjusted_quantity < rules.min_qty {
            adjusted_quantity = rules.min_qty;
        }

        // 再次检查名义金额 ≥ 5 USDT，必要时提升并对齐
        if adjusted_quantity * current_price < 5.0 {
            let needed_qty = 5.0 / current_price;
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
        let url = format!(
            "{}/fapi/v1/order?{}&signature={}",
            self.base_url, query, signature
        );

        let client = reqwest::Client::new();
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

        let client = reqwest::Client::new();
        let response: serde_json::Value = client.get(&url).send().await?.json().await?;

        let price: f64 = response["price"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("价格解析失败"))?
            .parse()?;

        Ok(price)
    }

    pub async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        // 先查缓存
        if let Some(cached) = self
            .rules_cache
            .read()
            .await
            .get(symbol)
            .cloned()
        {
            return Ok(cached);
        }

        // 未命中则请求并写入缓存
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);
        let client = reqwest::Client::new();
        let response: ExchangeInfo = client.get(&url).send().await?.json().await?;

        for symbol_info in response.symbols {
            if symbol_info.symbol == symbol {
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

                        let rules = TradingRules {
                            step_size,
                            min_qty,
                            quantity_precision: symbol_info.quantityPrecision,
                        };

                        self.rules_cache
                            .write()
                            .await
                            .insert(symbol.to_string(), rules.clone());

                        return Ok(rules);
                    }
                }
                return Err(anyhow::anyhow!("未找到LOT_SIZE filter: {}", symbol));
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

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await?;
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
}
