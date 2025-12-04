// OKX 交易所客户端实现
use crate::exchanges::traits::*;
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
pub struct OkxClient {
    api_key: String,
    secret_key: String,
    passphrase: String,
    base_url: String,
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

impl OkxClient {
    pub fn new(api_key: String, secret_key: String, passphrase: String, _testnet: bool) -> Self {
        let base_url = "https://www.okx.com".to_string();

        Self {
            api_key,
            secret_key,
            passphrase,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 转换交易对格式: BTCUSDT -> BTC-USDT-SWAP
    fn format_symbol(&self, symbol: &str) -> String {
        if symbol.contains('-') {
            symbol.to_string()
        } else {
            symbol.replace("USDT", "-USDT-SWAP")
        }
    }

    /// 反向转换: BTC-USDT-SWAP -> BTCUSDT
    fn unformat_symbol(&self, symbol: &str) -> String {
        symbol.replace("-USDT-SWAP", "USDT").replace("-", "")
    }

    /// OKX 签名方法
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
        headers.insert("OK-ACCESS-KEY", self.api_key.parse().unwrap());
        headers.insert(
            "OK-ACCESS-SIGN",
            self.sign(timestamp, method, request_path, body)
                .parse()
                .unwrap(),
        );
        headers.insert("OK-ACCESS-TIMESTAMP", timestamp.parse().unwrap());
        headers.insert("OK-ACCESS-PASSPHRASE", self.passphrase.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// 创建强制使用 IPv4 的 HTTP 客户端
    fn create_ipv4_client(&self) -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder()
            .local_address(Some(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)))
            .build()?)
    }

    /// 获取指定交易对的持仓信息
    pub async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        Ok(positions.into_iter().find(|p| p.symbol == symbol))
    }
}

#[async_trait]
impl ExchangeClient for OkxClient {
    fn get_exchange_name(&self) -> &str {
        "OKX"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/account/positions?instType=SWAP";
        let headers = self.build_headers(&timestamp, "GET", request_path, "");

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("OKX 获取持仓失败: {}", body);
            return Err(anyhow!("OKX API错误: {}", body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<OkxPosition>>,
        }

        #[derive(Debug, Deserialize)]
        #[allow(non_snake_case)]
        struct OkxPosition {
            instId: String,
            posSide: String,
            pos: String,
            avgPx: String,
            markPx: String,
            upl: String,
            lever: String,
            imr: String,
        }

        let resp: OkxResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析OKX持仓响应失败: {}，响应内容: {}", e, body))?;

        if resp.code != "0" {
            return Err(anyhow!("OKX API错误: {}", resp.msg));
        }

        let positions = resp.data.unwrap_or_default();
        let mut result = Vec::new();

        for pos in positions {
            let size: f64 = pos.pos.parse().unwrap_or(0.0);
            if size == 0.0 {
                continue;
            }

            result.push(Position {
                symbol: self.unformat_symbol(&pos.instId),
                side: if pos.posSide == "long" {
                    "LONG".to_string()
                } else {
                    "SHORT".to_string()
                },
                size: size.abs(),
                entry_price: pos.avgPx.parse().unwrap_or(0.0),
                mark_price: pos.markPx.parse().unwrap_or(0.0),
                pnl: pos.upl.parse().unwrap_or(0.0),
                leverage: pos.lever.parse().unwrap_or(1),
                margin: pos.imr.parse().unwrap_or(0.0),
            });
        }

        Ok(result)
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/account/balance";
        let headers = self.build_headers(&timestamp, "GET", request_path, "");

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("OKX 获取账户信息失败: {}", body);
            return Err(anyhow!("OKX API错误: {}", body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<OkxAccount>>,
        }

        #[derive(Debug, Deserialize)]
        #[allow(non_snake_case)]
        struct OkxDetail {
            ccy: String,
            availBal: Option<String>,
            frozenBal: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        #[allow(non_snake_case)]
        struct OkxAccount {
            totalEq: Option<String>,
            availEq: Option<String>,
            upl: Option<String>,
            frozenBal: Option<String>,
            details: Option<Vec<OkxDetail>>,
        }

        let resp: OkxResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析OKX账户响应失败: {}，响应: {}", e, body))?;
        if resp.code != "0" {
            return Err(anyhow!("OKX API错误: {}", resp.msg));
        }

        let account = resp
            .data
            .and_then(|d| d.into_iter().next())
            .ok_or_else(|| anyhow!("OKX账户数据为空"))?;

        let mut _total_balance = 0.0;
        let mut available_balance = 0.0;
        let mut margin_used = 0.0;

        // 从 details 数组中汇总所有 USDT 相关币种余额
        if let Some(details) = &account.details {
            for detail in details {
                if detail.ccy == "USDT" || detail.ccy == "USDC" {
                    let avail = detail
                        .availBal
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let frozen = detail
                        .frozenBal
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let coin_total = avail + frozen;

                    if coin_total > 0.01 {
                        info!("OKX {} 余额: {:.2}", detail.ccy, coin_total);
                    }

                    available_balance += avail;
                    margin_used += frozen;
                }
            }
            _total_balance = available_balance + margin_used;
        } else {
            // 如果没有 details，尝试使用账户级别的字段
            _total_balance = account.totalEq.and_then(|s| s.parse().ok()).unwrap_or(0.0);
            available_balance = account
                .availEq
                .as_ref()
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            margin_used = account
                .frozenBal
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
        }

        Ok(AccountInfo {
            total_balance: _total_balance,
            available_balance,
            unrealized_pnl: account.upl.and_then(|s| s.parse().ok()).unwrap_or(0.0),
            margin_used,
        })
    }

    async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        let okx_symbol = self.format_symbol(symbol);
        let request_path = format!("/api/v5/market/ticker?instId={}", okx_symbol);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client.get(&url).send().await?;

        let body = response.text().await?;

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<serde_json::Value>>,
        }

        let resp: OkxResponse = serde_json::from_str(&body)?;
        if resp.code != "0" {
            return Err(anyhow!("OKX获取价格失败: {}", resp.msg));
        }

        let price: f64 = resp
            .data
            .and_then(|d| d.into_iter().next())
            .and_then(|v| v["last"].as_str().and_then(|s| s.parse().ok()))
            .ok_or_else(|| anyhow!("价格字段缺失"))?;

        Ok(price)
    }

    async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        {
            let cache = self.rules_cache.read().await;
            if let Some(rules) = cache.get(symbol) {
                return Ok(rules.clone());
            }
        }

        let okx_symbol = self.format_symbol(symbol);
        let request_path = format!(
            "/api/v5/public/instruments?instType=SWAP&instId={}",
            okx_symbol
        );

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client.get(&url).send().await?;

        let body = response.text().await?;

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            #[allow(dead_code)]
            code: String,
            data: Option<Vec<serde_json::Value>>,
        }

        let resp: OkxResponse = serde_json::from_str(&body)?;
        let data = resp
            .data
            .and_then(|d| d.into_iter().next())
            .ok_or_else(|| anyhow!("交易规则数据为空"))?;

        let rules = TradingRules {
            step_size: data["lotSz"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.001),
            min_qty: data["minSz"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            quantity_precision: 3,
            price_precision: 2,
            tick_size: 0.0001, // OKX 默认最小价格跳动
            min_notional: None,
        };

        {
            let mut cache = self.rules_cache.write().await;
            cache.insert(symbol.to_string(), rules.clone());
        }

        Ok(rules)
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/account/set-leverage";

        let okx_symbol = self.format_symbol(symbol);
        let body = json!({
            "instId": okx_symbol,
            "lever": leverage.to_string(),
            "mgnMode": "cross",
        });

        let body_str = serde_json::to_string(&body)?;
        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            error!("OKX 设置杠杆失败: {}", body);
            return Err(anyhow!("OKX 设置杠杆失败: {}", body));
        }

        info!("✅ OKX设置杠杆成功: {}x", leverage);
        Ok(())
    }

    async fn set_margin_type(&self, _symbol: &str, _margin_type: &str) -> Result<()> {
        // OKX 在下单时指定 tdMode，不需要单独设置
        Ok(())
    }

    async fn set_position_mode(&self, dual_side: bool) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/account/set-position-mode";

        let body = json!({
            "posMode": if dual_side { "long_short_mode" } else { "net_mode" }
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            if body.contains("Position mode is already") {
                return Ok(());
            }
            warn!("OKX设置持仓模式警告: {}", body);
            return Ok(());
        }

        info!(
            "✅ OKX设置持仓模式成功: {}",
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
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/trade/order";

        let okx_symbol = self.format_symbol(symbol);
        let body = json!({
            "instId": okx_symbol,
            "tdMode": "cross",
            "side": "buy",
            "posSide": "long",
            "ordType": "market",
            "sz": quantity.to_string(),
        });

        let body_str = serde_json::to_string(&body)?;
        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("OKX开多失败: {}", resp_body);
            return Err(anyhow!("OKX开多失败: {}", resp_body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<serde_json::Value>>,
        }

        let resp: OkxResponse = serde_json::from_str(&resp_body)?;
        if resp.code != "0" {
            return Err(anyhow!("OKX开多失败: {}", resp.msg));
        }

        info!("✅ OKX开多成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d.into_iter().next())
                .and_then(|v| v["ordId"].as_str().map(|s| s.to_string()))
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
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/trade/order";

        let okx_symbol = self.format_symbol(symbol);
        let body = json!({
            "instId": okx_symbol,
            "tdMode": "cross",
            "side": "sell",
            "posSide": "short",
            "ordType": "market",
            "sz": quantity.to_string(),
        });

        let body_str = serde_json::to_string(&body)?;
        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("OKX开空失败: {}", resp_body);
            return Err(anyhow!("OKX开空失败: {}", resp_body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<serde_json::Value>>,
        }

        let resp: OkxResponse = serde_json::from_str(&resp_body)?;
        if resp.code != "0" {
            return Err(anyhow!("OKX开空失败: {}", resp.msg));
        }

        info!("✅ OKX开空成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d.into_iter().next())
                .and_then(|v| v["ordId"].as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            symbol: symbol.to_string(),
            side: "SELL".to_string(),
            quantity,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }

    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let request_path = "/api/v5/trade/order";

        let okx_symbol = self.format_symbol(symbol);
        let (order_side, pos_side) = if side == "LONG" {
            ("sell", "long")
        } else {
            ("buy", "short")
        };

        let body = json!({
            "instId": okx_symbol,
            "tdMode": "cross",
            "side": order_side,
            "posSide": pos_side,
            "ordType": "market",
            "sz": size.to_string(),
            "reduceOnly": true,
        });

        let body_str = serde_json::to_string(&body)?;
        let headers = self.build_headers(&timestamp, "POST", request_path, &body_str);

        let url = format!("{}{}", self.base_url, request_path);
        let client = self.create_ipv4_client()?;
        let response = client
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        let resp_body = response.text().await?;

        if !status.is_success() {
            error!("OKX平仓失败: {}", resp_body);
            return Err(anyhow!("OKX平仓失败: {}", resp_body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxResponse {
            code: String,
            msg: String,
            data: Option<Vec<serde_json::Value>>,
        }

        let resp: OkxResponse = serde_json::from_str(&resp_body)?;
        if resp.code != "0" {
            return Err(anyhow!("OKX平仓失败: {}", resp.msg));
        }

        info!("✅ OKX平仓成功: {} {} {}", symbol, side, size);
        Ok(OrderResult {
            order_id: resp
                .data
                .and_then(|d| d.into_iter().next())
                .and_then(|v| v["ordId"].as_str().map(|s| s.to_string()))
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
        // 调用 OKX 行情接口并按统一格式输出数值 K 线
        let limit_value = limit.unwrap_or(100);
        let okx_symbol = self.format_symbol(symbol);
        let request_path = format!(
            "/api/v5/market/candles?instId={}&bar={}&limit={}",
            okx_symbol, interval, limit_value
        );
        let url = format!("{}{}", self.base_url, request_path);

        let client = self.create_ipv4_client()?;
        let response = client.get(&url).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("OKX 获取K线失败: {}", body);
            return Err(anyhow!("OKX API错误: {}", body));
        }

        #[derive(Debug, Deserialize)]
        struct OkxKlineResponse {
            code: String,
            msg: String,
            data: Option<Vec<Vec<String>>>,
        }

        let resp: OkxKlineResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析OKX K线响应失败: {}，响应内容: {}", e, body))?;

        if resp.code != "0" {
            return Err(anyhow!("OKX API错误: {}", resp.msg));
        }

        let klines = resp
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|entry| {
                // OKX 所有字段均为字符串，这里取前六项并转换为 f64
                let parse = |idx: usize| -> f64 {
                    entry
                        .get(idx)
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0)
                };
                vec![parse(0), parse(1), parse(2), parse(3), parse(4), parse(5)]
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
        // quantity_delta > 0 视为加仓，否则走减仓逻辑
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
