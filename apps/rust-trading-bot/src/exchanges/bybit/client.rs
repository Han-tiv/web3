// Bybit 交易所客户端实现
use crate::exchanges::traits::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
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
pub struct BybitClient {
    api_key: String,
    secret_key: String,
    base_url: String,
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

#[derive(Debug, Deserialize)]
struct BybitResponse<T> {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct BybitPositionList {
    list: Vec<BybitPosition>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BybitPosition {
    symbol: String,
    side: String,     // "Buy" or "Sell"
    size: String,     // 持仓数量
    avgPrice: String, // 开仓均价
    markPrice: String,
    unrealisedPnl: String,
    leverage: String,
    positionIM: String, // 仓位保证金
}

#[derive(Debug, Deserialize)]
struct BybitAccountList {
    list: Vec<BybitAccount>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BybitAccount {
    totalEquity: String,
    totalAvailableBalance: String,
    totalPerpUPL: String,
    totalInitialMargin: String,
}

#[derive(Debug, Deserialize)]
struct BybitKlineResult {
    list: Vec<Vec<String>>,
}

impl BybitClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            "https://api.bybit.com".to_string()
        };

        Self {
            api_key,
            secret_key,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Bybit 签名方法
    fn sign(&self, timestamp: &str, params: &str, recv_window: &str) -> String {
        let prehash = format!("{}{}{}{}", timestamp, &self.api_key, recv_window, params);
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(prehash.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// 构建请求头
    fn build_headers(
        &self,
        timestamp: &str,
        params: &str,
        recv_window: &str,
    ) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-BAPI-API-KEY", self.api_key.parse().unwrap());
        headers.insert(
            "X-BAPI-SIGN",
            self.sign(timestamp, params, recv_window).parse().unwrap(),
        );
        headers.insert("X-BAPI-TIMESTAMP", timestamp.parse().unwrap());
        headers.insert("X-BAPI-RECV-WINDOW", recv_window.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// 获取指定交易对的持仓信息
    pub async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }
}

#[async_trait]
impl ExchangeClient for BybitClient {
    fn get_exchange_name(&self) -> &str {
        "Bybit"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";
        let params = "category=linear&settleCoin=USDT".to_string();
        let headers = self.build_headers(&timestamp, &params, recv_window);

        let url = format!("{}/v5/position/list?{}", self.base_url, params);
        let client = reqwest::Client::new();
        let response = client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("Bybit 获取持仓失败: {}", body);
            return Err(anyhow!("Bybit API错误: {}", body));
        }

        let resp: BybitResponse<BybitPositionList> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析Bybit持仓响应失败: {}，响应内容: {}", e, body))?;

        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit API错误: {}", resp.ret_msg));
        }

        let positions = resp.result.map(|r| r.list).unwrap_or_default();
        let mut result = Vec::new();

        for pos in positions {
            let size: f64 = pos.size.parse().unwrap_or(0.0);
            if size == 0.0 {
                continue;
            }

            result.push(Position {
                symbol: pos.symbol,
                side: if pos.side == "Buy" {
                    "LONG".to_string()
                } else {
                    "SHORT".to_string()
                },
                size: size.abs(),
                entry_price: pos.avgPrice.parse().unwrap_or(0.0),
                mark_price: pos.markPrice.parse().unwrap_or(0.0),
                pnl: pos.unrealisedPnl.parse().unwrap_or(0.0),
                leverage: pos.leverage.parse().unwrap_or(1),
                margin: pos.positionIM.parse().unwrap_or(0.0),
            });
        }

        Ok(result)
    }

    async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";

        // 查询所有账户类型
        let account_types = vec!["UNIFIED", "CONTRACT", "SPOT", "FUND"];
        let mut total_balance = 0.0;
        let mut available_balance = 0.0;
        let mut unrealized_pnl = 0.0;
        let mut margin_used = 0.0;

        for account_type in account_types {
            let params = format!("accountType={}", account_type);
            let headers = self.build_headers(&timestamp, &params, recv_window);

            let url = format!("{}/v5/account/wallet-balance?{}", self.base_url, params);
            let client = reqwest::Client::new();
            let response = client.get(&url).headers(headers).send().await?;

            let status = response.status();
            let body = response.text().await?;

            if !status.is_success() {
                // 某些账户类型可能不存在，跳过
                continue;
            }

            let resp: BybitResponse<BybitAccountList> = match serde_json::from_str(&body) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if resp.ret_code != 0 {
                continue;
            }

            if let Some(result) = resp.result {
                for account in result.list {
                    let equity = account.totalEquity.parse::<f64>().unwrap_or(0.0);
                    let avail = account.totalAvailableBalance.parse::<f64>().unwrap_or(0.0);
                    let pnl = account.totalPerpUPL.parse::<f64>().unwrap_or(0.0);
                    let margin = account.totalInitialMargin.parse::<f64>().unwrap_or(0.0);

                    if equity > 0.01 {
                        info!("Bybit {} 账户: 余额={:.2} USDT", account_type, equity);
                    }

                    total_balance += equity;
                    available_balance += avail;
                    unrealized_pnl += pnl;
                    margin_used += margin;
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
        let url = format!(
            "{}/v5/market/tickers?category=linear&symbol={}",
            self.base_url, symbol
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit获取价格失败: {}", resp.ret_msg));
        }

        let data = resp.result.ok_or_else(|| anyhow!("价格数据为空"))?;
        let price: f64 = data["list"][0]["lastPrice"]
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

        let url = format!(
            "{}/v5/market/instruments-info?category=linear&symbol={}",
            self.base_url, symbol
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit获取交易规则失败: {}", resp.ret_msg));
        }

        let data = resp.result.ok_or_else(|| anyhow!("交易规则数据为空"))?;
        let instrument = &data["list"][0];

        let lot_size_filter = &instrument["lotSizeFilter"];
        let rules = TradingRules {
            step_size: lot_size_filter["qtyStep"]
                .as_str()
                .unwrap_or("0.001")
                .parse()
                .unwrap_or(0.001),
            min_qty: lot_size_filter["minOrderQty"]
                .as_str()
                .unwrap_or("0.001")
                .parse()
                .unwrap_or(0.001),
            quantity_precision: 3, // Bybit 通常是 3 位
            price_precision: 2,
            tick_size: 0.0001, // Bybit 默认价格步长
            min_notional: None,
        };

        // 缓存规则
        {
            let mut cache = self.rules_cache.write().await;
            cache.insert(symbol.to_string(), rules.clone());
        }

        Ok(rules)
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";

        let body = json!({
            "category": "linear",
            "symbol": symbol,
            "buyLeverage": leverage.to_string(),
            "sellLeverage": leverage.to_string()
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/position/set-leverage", self.base_url);
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
            error!("Bybit设置杠杆失败: {}", resp_body);
            return Err(anyhow!("Bybit设置杠杆失败: {}", resp_body));
        }

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit设置杠杆失败: {}", resp.ret_msg));
        }

        info!("✅ Bybit设置杠杆成功: {}x", leverage);
        Ok(())
    }

    async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";

        // Bybit: 0=全仓, 1=逐仓
        let trade_mode = if margin_type == "ISOLATED" { 1 } else { 0 };

        let body = json!({
            "category": "linear",
            "symbol": symbol,
            "tradeMode": trade_mode,
            "buyLeverage": "10",  // 需要提供杠杆值
            "sellLeverage": "10"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/position/switch-isolated", self.base_url);
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
            // Bybit 如果已经是该模式会报错，这是正常的
            if resp_body.contains("position idx was not modified") {
                return Ok(());
            }
            warn!("Bybit设置保证金模式警告: {}", resp_body);
            return Ok(()); // 不阻塞交易
        }

        info!("✅ Bybit设置保证金模式成功: {}", margin_type);
        Ok(())
    }

    async fn set_position_mode(&self, dual_side: bool) -> Result<()> {
        let timestamp = Utc::now().timestamp_millis().to_string();
        let recv_window = "5000";

        // Bybit: 0=单向持仓, 3=双向持仓
        let mode = if dual_side { 3 } else { 0 };

        let body = json!({
            "category": "linear",
            "mode": mode
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/position/switch-mode", self.base_url);
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
            if body.contains("position mode is not modified") {
                return Ok(());
            }
            warn!("Bybit设置持仓模式警告: {}", body);
            return Ok(()); // 不阻塞交易
        }

        info!(
            "✅ Bybit设置持仓模式成功: {}",
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
        let recv_window = "5000";

        let body = json!({
            "category": "linear",
            "symbol": symbol,
            "side": "Buy",
            "orderType": "Market",
            "qty": quantity.to_string(),
            "positionIdx": 0  // 0=单向持仓
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/order/create", self.base_url);
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
            error!("Bybit开多失败: {}", resp_body);
            return Err(anyhow!("Bybit开多失败: {}", resp_body));
        }

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit开多失败: {}", resp.ret_msg));
        }

        info!("✅ Bybit开多成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .result
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
        let recv_window = "5000";

        let body = json!({
            "category": "linear",
            "symbol": symbol,
            "side": "Sell",
            "orderType": "Market",
            "qty": quantity.to_string(),
            "positionIdx": 0
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/order/create", self.base_url);
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
            error!("Bybit开空失败: {}", resp_body);
            return Err(anyhow!("Bybit开空失败: {}", resp_body));
        }

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit开空失败: {}", resp.ret_msg));
        }

        info!("✅ Bybit开空成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .result
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
        let recv_window = "5000";

        // 平仓用反向订单
        let order_side = if side == "LONG" { "Sell" } else { "Buy" };

        let body = json!({
            "category": "linear",
            "symbol": symbol,
            "side": order_side,
            "orderType": "Market",
            "qty": size.to_string(),
            "reduceOnly": true,
            "positionIdx": 0
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, &body_str, recv_window);

        let url = format!("{}/v5/order/create", self.base_url);
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
            error!("Bybit平仓失败: {}", resp_body);
            return Err(anyhow!("Bybit平仓失败: {}", resp_body));
        }

        let resp: BybitResponse<serde_json::Value> = serde_json::from_str(&resp_body)?;
        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit平仓失败: {}", resp.ret_msg));
        }

        info!("✅ Bybit平仓成功: {} {} {}", symbol, side, size);
        Ok(OrderResult {
            order_id: resp
                .result
                .and_then(|d| d["orderId"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            symbol: symbol.to_string(),
            side: order_side.to_string(),
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
        let url = format!("{}/v5/market/kline", self.base_url);
        let params = [
            ("category", "linear".to_string()),
            ("symbol", symbol.to_string()),
            ("interval", interval.to_string()),
            ("limit", limit_value.to_string()),
        ];

        let client = reqwest::Client::new();
        let response = client.get(&url).query(&params).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("Bybit获取K线失败: {}", body);
            return Err(anyhow!("Bybit获取K线失败: {}", body));
        }

        let resp: BybitResponse<BybitKlineResult> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析Bybit K线响应失败: {}，响应内容: {}", e, body))?;

        if resp.ret_code != 0 {
            return Err(anyhow!("Bybit获取K线失败: {}", resp.ret_msg));
        }

        let result = resp
            .result
            .ok_or_else(|| anyhow!("Bybit返回的K线数据为空"))?;

        let klines = result
            .list
            .into_iter()
            .map(|fields| -> Result<Vec<f64>> {
                if fields.len() < 6 {
                    return Err(anyhow!("Bybit K线字段不足: {:?}", fields));
                }

                let parse_value = |idx: usize, field: &str| -> Result<f64> {
                    let raw = fields
                        .get(idx)
                        .ok_or_else(|| anyhow!("Bybit K线缺少 {} 字段", field))?;
                    raw.parse::<f64>().map_err(|e| {
                        anyhow!("Bybit K线字段 {} 解析失败: {} (值: {})", field, e, raw)
                    })
                };

                // Bybit 返回的第七位为成交额，这里仅取前六项
                let timestamp = parse_value(0, "timestamp")?;
                let open = parse_value(1, "open")?;
                let high = parse_value(2, "high")?;
                let low = parse_value(3, "low")?;
                let close = parse_value(4, "close")?;
                let volume = parse_value(5, "volume")?;

                Ok(vec![timestamp, open, high, low, close, volume])
            })
            .collect::<Result<Vec<_>>>()?;

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
        if quantity_delta == 0.0 {
            return Err(anyhow!("调整数量为 0，不执行操作"));
        }

        if side != "LONG" && side != "SHORT" {
            return Err(anyhow!("未知持仓方向: {}", side));
        }

        if quantity_delta > 0.0 {
            if side == "LONG" {
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
}
