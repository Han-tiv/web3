// Bitget 交易所客户端实现
use crate::exchange_trait::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use log::{error, info, warn};
use reqwest;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct BitgetClient {
    api_key: String,
    secret_key: String,
    passphrase: String,
    base_url: String,
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

#[derive(Debug, Deserialize)]
struct BitgetResponse<T> {
    code: String,
    msg: String,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BitgetPosition {
    symbol: String,
    holdSide: String, // "long" or "short"
    total: String,    // 持仓数量
    available: String,
    openPriceAvg: String, // 开仓均价
    markPrice: String,
    unrealizedPL: String, // 未实现盈亏
    leverage: String,
    margin: String, // 保证金
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BitgetAccount {
    marginCoin: String,   // 保证金币种
    usdtEquity: String,   // 总权益
    available: String,    // 可用余额
    locked: String,       // 已用保证金
    unrealizedPL: String, // 未实现盈亏
}

impl BitgetClient {
    pub fn new(api_key: String, secret_key: String, passphrase: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://api.bitget.com".to_string() // Bitget 测试环境需要特殊申请
        } else {
            "https://api.bitget.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            passphrase,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Bitget 签名方法
    fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> String {
        let prehash = format!("{}{}{}{}", timestamp, method, request_path, body);
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(prehash.as_bytes());
        general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    /// 构建请求头
    fn build_headers(
        &self,
        timestamp: &str,
        method: &str,
        request_path: &str,
        body: &str,
    ) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("ACCESS-KEY", self.api_key.parse().unwrap());
        headers.insert(
            "ACCESS-SIGN",
            self.sign(timestamp, method, request_path, body)
                .parse()
                .unwrap(),
        );
        headers.insert("ACCESS-TIMESTAMP", timestamp.parse().unwrap());
        headers.insert("ACCESS-PASSPHRASE", self.passphrase.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("locale", "zh-CN".parse().unwrap());
        headers
    }

    /// 转换交易对格式: BTCUSDT -> BTCUSDT_UMCBL
    fn format_symbol(&self, symbol: &str) -> String {
        if symbol.ends_with("_UMCBL") {
            symbol.to_string()
        } else {
            format!("{}_UMCBL", symbol)
        }
    }

    /// 反向转换: BTCUSDT_UMCBL -> BTCUSDT
    fn unformat_symbol(&self, symbol: &str) -> String {
        symbol.replace("_UMCBL", "")
    }

    /// 获取指定交易对的持仓信息
    pub async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }
}

#[async_trait]
impl ExchangeClient for BitgetClient {
    fn get_exchange_name(&self) -> &str {
        "Bitget"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/position/allPosition?productType=umcbl";
        let headers = self.build_headers(&timestamp, "GET", request_path, "");

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("Bitget 获取持仓失败: {}", body);
            return Err(anyhow!("Bitget API错误: {}", body));
        }

        let resp: BitgetResponse<Vec<BitgetPosition>> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析Bitget持仓响应失败: {}，响应内容: {}", e, body))?;

        if resp.code != "00000" {
            return Err(anyhow!("Bitget API错误: {}", resp.msg));
        }

        let positions = resp.data.unwrap_or_default();
        let mut result = Vec::new();

        for pos in positions {
            let size: f64 = pos.total.parse().unwrap_or(0.0);
            if size == 0.0 {
                continue;
            }

            result.push(Position {
                symbol: self.unformat_symbol(&pos.symbol),
                side: if pos.holdSide == "long" {
                    "LONG".to_string()
                } else {
                    "SHORT".to_string()
                },
                size: size.abs(),
                entry_price: pos.openPriceAvg.parse().unwrap_or(0.0),
                mark_price: pos.markPrice.parse().unwrap_or(0.0),
                pnl: pos.unrealizedPL.parse().unwrap_or(0.0),
                leverage: pos.leverage.parse().unwrap_or(1),
                margin: pos.margin.parse().unwrap_or(0.0),
            });
        }

        Ok(result)
    }

    async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let client = reqwest::Client::new();
        let mut total_balance = 0.0;
        let mut available_balance = 0.0;
        let mut unrealized_pnl = 0.0;
        let mut margin_used = 0.0;

        // 1. 查询合约账户 (U本位)
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/account/accounts?productType=umcbl";
        let headers = self.build_headers(&timestamp, "GET", request_path, "");
        let url = format!("{}{}", self.base_url, request_path);

        let response = client.get(&url).headers(headers).send().await?;
        if response.status().is_success() {
            let body = response.text().await?;
            let resp: BitgetResponse<Vec<BitgetAccount>> = serde_json::from_str(&body)
                .map_err(|e| anyhow!("解析Bitget账户响应失败: {}，响应内容: {}", e, body))?;

            if resp.code == "00000" {
                if let Some(accounts) = resp.data {
                    for account in accounts {
                        if account.marginCoin == "USDT" || account.marginCoin == "USDC" {
                            let equity = account.usdtEquity.parse::<f64>().unwrap_or(0.0);
                            if equity > 0.01 {
                                info!("Bitget 合约账户 {}: {:.2}", account.marginCoin, equity);
                            }
                            total_balance += equity;
                            available_balance += account.available.parse::<f64>().unwrap_or(0.0);
                            unrealized_pnl += account.unrealizedPL.parse::<f64>().unwrap_or(0.0);
                            margin_used += account.locked.parse::<f64>().unwrap_or(0.0);
                        }
                    }
                }
            }
        }

        // 2. 查询现货账户
        let timestamp2 = Utc::now().timestamp_millis().to_string();
        let spot_path = "/api/spot/v1/account/assets";
        let spot_headers = self.build_headers(&timestamp2, "GET", spot_path, "");
        let spot_url = format!("{}{}", self.base_url, spot_path);

        if let Ok(spot_response) = client.get(&spot_url).headers(spot_headers).send().await {
            if spot_response.status().is_success() {
                if let Ok(spot_body) = spot_response.text().await {
                    #[derive(Debug, Deserialize)]
                    struct SpotAsset {
                        coinName: String,
                        available: String,
                        frozen: String,
                    }

                    if let Ok(spot_resp) =
                        serde_json::from_str::<BitgetResponse<Vec<SpotAsset>>>(&spot_body)
                    {
                        if spot_resp.code == "00000" {
                            if let Some(assets) = spot_resp.data {
                                for asset in assets {
                                    if asset.coinName == "USDT" || asset.coinName == "USDC" {
                                        let avail = asset.available.parse::<f64>().unwrap_or(0.0);
                                        let frozen = asset.frozen.parse::<f64>().unwrap_or(0.0);
                                        let spot_total = avail + frozen;

                                        if spot_total > 0.01 {
                                            info!(
                                                "Bitget 现货账户 {}: {:.2}",
                                                asset.coinName, spot_total
                                            );
                                            total_balance += spot_total;
                                            available_balance += avail;
                                            margin_used += frozen;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(AccountInfo {
            total_balance,
            available_balance,
            unrealized_pnl,
            margin_used,
        })
    }

    async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        let bitget_symbol = self.format_symbol(symbol);
        let url = format!(
            "{}/api/mix/v1/market/ticker?symbol={}",
            self.base_url, bitget_symbol
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let resp: BitgetResponse<serde_json::Value> = serde_json::from_str(&body)?;
        if resp.code != "00000" {
            return Err(anyhow!("Bitget获取价格失败: {}", resp.msg));
        }

        let data = resp.data.ok_or_else(|| anyhow!("价格数据为空"))?;
        let price: f64 = data["last"]
            .as_str()
            .ok_or_else(|| anyhow!("价格字段缺失"))?
            .parse()?;

        Ok(price)
    }

    async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        // 检查缓存
        {
            let cache = self.rules_cache.read().await;
            if let Some(rules) = cache.get(symbol) {
                return Ok(rules.clone());
            }
        }

        let bitget_symbol = self.format_symbol(symbol);
        let url = format!(
            "{}/api/mix/v1/market/contracts?productType=umcbl",
            self.base_url
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let resp: BitgetResponse<Vec<serde_json::Value>> = serde_json::from_str(&body)?;
        if resp.code != "00000" {
            return Err(anyhow!("Bitget获取交易规则失败: {}", resp.msg));
        }

        let contracts = resp.data.ok_or_else(|| anyhow!("合约数据为空"))?;
        let contract = contracts
            .iter()
            .find(|c| c["symbol"].as_str() == Some(&bitget_symbol))
            .ok_or_else(|| anyhow!("未找到交易对: {}", bitget_symbol))?;

        let rules = TradingRules {
            step_size: contract["sizeMultiplier"]
                .as_str()
                .unwrap_or("0.001")
                .parse()
                .unwrap_or(0.001),
            min_qty: contract["minTradeNum"]
                .as_str()
                .unwrap_or("1")
                .parse()
                .unwrap_or(1.0),
            quantity_precision: contract["volumePlace"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0),
            price_precision: contract["pricePlace"]
                .as_str()
                .unwrap_or("2")
                .parse()
                .unwrap_or(2),
            tick_size: 0.0001, // Bitget 默认最小价格跳动
        };

        // 缓存规则
        {
            let mut cache = self.rules_cache.write().await;
            cache.insert(symbol.to_string(), rules.clone());
        }

        Ok(rules)
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Vec<f64>>> {
        let granularity = match interval {
            "1m" => 60,
            "5m" => 300,
            "15m" => 900,
            "30m" => 1800,
            "1h" => 3_600,
            "4h" => 14_400,
            "1d" => 86_400,
            _ => return Err(anyhow!("Bitget暂不支持该周期: {}", interval)),
        };

        let bitget_symbol = self.format_symbol(symbol);
        let limit = limit.unwrap_or(100);
        let url = format!(
            "{}/api/mix/v1/market/candles?symbol={}&granularity={}&limit={}",
            self.base_url, bitget_symbol, granularity, limit
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let resp: BitgetResponse<Vec<Vec<String>>> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析Bitget K线响应失败: {}，响应内容: {}", e, body))?;

        if resp.code != "00000" {
            return Err(anyhow!("Bitget获取K线失败: {}", resp.msg));
        }

        let mut klines = Vec::new();
        if let Some(data) = resp.data {
            for entry in data {
                if entry.len() < 6 {
                    continue;
                }

                let timestamp = entry
                    .get(0)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();
                let open = entry
                    .get(1)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();
                let high = entry
                    .get(2)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();
                let low = entry
                    .get(3)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();
                let close = entry
                    .get(4)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();
                let volume = entry
                    .get(5)
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or_default();

                klines.push(vec![timestamp, open, high, low, close, volume]);
            }
        }

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
            if side.eq_ignore_ascii_case("LONG") {
                self.open_long(symbol, quantity_delta, leverage, margin_type, false)
                    .await
            } else {
                self.open_short(symbol, quantity_delta, leverage, margin_type, false)
                    .await
            }
        } else {
            let reduce_amount = quantity_delta.abs();
            self.close_position(symbol, side, reduce_amount).await
        }
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/account/setLeverage";

        let bitget_symbol = self.format_symbol(symbol);
        let body = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "leverage": leverage.to_string(),
            "holdSide": "long"  // Bitget需要分别设置多空杠杆
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers.clone())
            .body(body_str.clone())
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("Bitget设置多头杠杆失败: {}", resp_body);
            return Err(anyhow!("Bitget设置多头杠杆失败: {}", resp_body));
        }

        // 再设置空头杠杆
        let timestamp2 = Utc::now().timestamp_millis().to_string();
        let body2 = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "leverage": leverage.to_string(),
            "holdSide": "short"
        });
        let body_str2 = serde_json::to_string(&body2)?;
        let headers2 = self.build_headers(&timestamp2, "POST", request_path, &body_str2);

        let response2 = client
            .post(&url)
            .headers(headers2)
            .body(body_str2)
            .send()
            .await?;

        let status2 = response2.status();
        let resp_body2 = response2.text().await?;

        if !status2.is_success() {
            warn!("Bitget设置空头杠杆失败: {}", resp_body2);
        }

        info!("✅ Bitget设置杠杆成功: {}x", leverage);
        Ok(())
    }

    async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/account/setMarginMode";

        let bitget_symbol = self.format_symbol(symbol);
        let bitget_margin_type = if margin_type == "ISOLATED" {
            "fixed"
        } else {
            "crossed"
        };

        let body = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "marginMode": bitget_margin_type
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            // Bitget在已设置相同保证金模式时会报错，这是正常的
            if resp_body.contains("The margin mode is already this mode") {
                return Ok(());
            }
            error!("Bitget设置保证金模式失败: {}", resp_body);
            return Err(anyhow!("Bitget设置保证金模式失败: {}", resp_body));
        }

        info!("✅ Bitget设置保证金模式成功: {}", margin_type);
        Ok(())
    }

    async fn set_position_mode(&self, dual_side: bool) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/account/setPositionMode";

        let body = json!({
            "productType": "umcbl",
            "holdMode": if dual_side { "double_hold" } else { "single_hold" }
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            // 如果已经是该模式，不报错
            if body.contains("already") {
                return Ok(());
            }
            error!("Bitget设置持仓模式失败: {}", body);
            return Err(anyhow!("Bitget设置持仓模式失败: {}", body));
        }

        info!(
            "✅ Bitget设置持仓模式成功: {}",
            if dual_side { "双向" } else { "单向" }
        );
        Ok(())
    }

    async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        _leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/order/placeOrder";

        let bitget_symbol = self.format_symbol(symbol);
        let body = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "size": quantity.to_string(),
            "side": "open_long",
            "orderType": "market",
            "timeInForceValue": "normal"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("Bitget开多失败: {}", resp_body);
            return Err(anyhow!("Bitget开多失败: {}", resp_body));
        }

        let resp: BitgetResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.code != "00000" {
            return Err(anyhow!("Bitget开多失败: {}", resp.msg));
        }

        info!("✅ Bitget开多成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d["orderId"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            symbol: symbol.to_string(),
            side: "BUY".to_string(),
            quantity,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }

    async fn open_short(
        &self,
        symbol: &str,
        quantity: f64,
        _leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/order/placeOrder";

        let bitget_symbol = self.format_symbol(symbol);
        let body = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "size": quantity.to_string(),
            "side": "open_short",
            "orderType": "market",
            "timeInForceValue": "normal"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("Bitget开空失败: {}", resp_body);
            return Err(anyhow!("Bitget开空失败: {}", resp_body));
        }

        let resp: BitgetResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.code != "00000" {
            return Err(anyhow!("Bitget开空失败: {}", resp.msg));
        }

        info!("✅ Bitget开空成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d["orderId"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            symbol: symbol.to_string(),
            side: "SELL".to_string(),
            quantity,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }

    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let request_path = "/api/mix/v1/order/placeOrder";

        let bitget_symbol = self.format_symbol(symbol);
        let bitget_side = if side == "LONG" {
            "close_long"
        } else {
            "close_short"
        };

        let body = json!({
            "symbol": bitget_symbol,
            "marginCoin": "USDT",
            "size": size.to_string(),
            "side": bitget_side,
            "orderType": "market",
            "timeInForceValue": "normal"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("Bitget平仓失败: {}", resp_body);
            return Err(anyhow!("Bitget平仓失败: {}", resp_body));
        }

        let resp: BitgetResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.code != "00000" {
            return Err(anyhow!("Bitget平仓失败: {}", resp.msg));
        }

        info!("✅ Bitget平仓成功: {} {} {}", symbol, side, size);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d["orderId"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            symbol: symbol.to_string(),
            side: if side == "LONG" {
                "SELL".to_string()
            } else {
                "BUY".to_string()
            },
            quantity: size,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }
}
