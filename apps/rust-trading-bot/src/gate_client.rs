// Gate.io 交易所客户端实现
use crate::exchange_trait::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use log::{error, info, warn};
use reqwest;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha512;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Clone)]
pub struct GateClient {
    api_key: String,
    secret_key: String,
    base_url: String,
    settle: String,  // USDT or USD
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GatePosition {
    contract: String,
    size: i64,             // 持仓数量，正数多仓，负数空仓
    entry_price: String,
    mark_price: String,
    unrealised_pnl: String,
    leverage: String,
    margin: String,
}

#[derive(Debug, Deserialize)]
struct GateAccount {
    total: String,
    unrealised_pnl: String,
    available: String,
    position_margin: String,
}

impl GateClient {
    pub fn new(api_key: String, secret_key: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://fx-api-testnet.gateio.ws".to_string()
        } else {
            "https://api.gateio.ws".to_string()
        };

        Self {
            api_key,
            secret_key,
            base_url,
            settle: "usdt".to_string(),
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gate 签名方法
    fn sign(&self, method: &str, url_path: &str, query_string: &str, payload_hash: &str, timestamp: &str) -> String {
        let prehash = format!("{}\n{}\n{}\n{}\n{}", method, url_path, query_string, payload_hash, timestamp);
        let mut mac = HmacSha512::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(prehash.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn hash_payload(&self, body: &str) -> String {
        use sha2::Digest;
        let mut hasher = Sha512::new();
        hasher.update(body.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 构建请求头
    fn build_headers(
        &self,
        method: &str,
        url_path: &str,
        query_string: &str,
        body: &str,
    ) -> reqwest::header::HeaderMap {
        let timestamp = Utc::now().timestamp().to_string();
        let payload_hash = self.hash_payload(body);
        let signature = self.sign(method, url_path, query_string, &payload_hash, &timestamp);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("KEY", self.api_key.parse().unwrap());
        headers.insert("SIGN", signature.parse().unwrap());
        headers.insert("Timestamp", timestamp.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// 转换交易对格式: BTCUSDT -> BTC_USDT
    fn format_symbol(&self, symbol: &str) -> String {
        if symbol.contains('_') {
            symbol.to_string()
        } else {
            symbol.replace("USDT", "_USDT")
        }
    }

    /// 反向转换: BTC_USDT -> BTCUSDT
    fn unformat_symbol(&self, symbol: &str) -> String {
        symbol.replace("_", "")
    }
}

#[async_trait]
impl ExchangeClient for GateClient {
    fn get_exchange_name(&self) -> &str {
        "Gate"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let url_path = format!("/api/v4/futures/{}/positions", self.settle);
        let query_string = "";
        let headers = self.build_headers("GET", &url_path, query_string, "");

        let url = format!("{}{}", self.base_url, url_path);
        let client = reqwest::Client::new();
        let response = client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            error!("Gate 获取持仓失败: {}", body);
            return Err(anyhow!("Gate API错误: {}", body));
        }

        let positions: Vec<GatePosition> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("解析Gate持仓响应失败: {}，响应内容: {}", e, body))?;

        let mut result = Vec::new();
        for pos in positions {
            if pos.size == 0 {
                continue;
            }

            result.push(Position {
                symbol: self.unformat_symbol(&pos.contract),
                side: if pos.size > 0 {
                    "LONG".to_string()
                } else {
                    "SHORT".to_string()
                },
                size: pos.size.abs() as f64,
                entry_price: pos.entry_price.parse().unwrap_or(0.0),
                mark_price: pos.mark_price.parse().unwrap_or(0.0),
                pnl: pos.unrealised_pnl.parse().unwrap_or(0.0),
                leverage: pos.leverage.parse().unwrap_or(1),
                margin: pos.margin.parse().unwrap_or(0.0),
            });
        }

        Ok(result)
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let client = reqwest::Client::new();
        let mut total_balance = 0.0;
        let mut available_balance = 0.0;
        let mut unrealized_pnl = 0.0;
        let mut margin_used = 0.0;

        // 1. 查询合约账户
        let url_path = format!("/api/v4/futures/{}/accounts", self.settle);
        let query_string = "";
        let headers = self.build_headers("GET", &url_path, query_string, "");
        let url = format!("{}{}", self.base_url, url_path);
        
        let response = client.get(&url).headers(headers).send().await?;
        if response.status().is_success() {
            if let Ok(body) = response.text().await {
                if let Ok(account) = serde_json::from_str::<GateAccount>(&body) {
                    let futures_total = account.available.parse::<f64>().unwrap_or(0.0) 
                        + account.position_margin.parse::<f64>().unwrap_or(0.0);
                    
                    if futures_total > 0.01 {
                        info!("Gate 合约账户: {:.2} USDT", futures_total);
                    }
                    
                    total_balance += futures_total;
                    available_balance += account.available.parse::<f64>().unwrap_or(0.0);
                    unrealized_pnl += account.unrealised_pnl.parse::<f64>().unwrap_or(0.0);
                    margin_used += account.position_margin.parse::<f64>().unwrap_or(0.0);
                }
            }
        }

        // 2. 查询现货账户
        let spot_path = "/api/v4/spot/accounts";
        let spot_headers = self.build_headers("GET", spot_path, "", "");
        let spot_url = format!("{}{}", self.base_url, spot_path);

        if let Ok(spot_response) = client.get(&spot_url).headers(spot_headers).send().await {
            if spot_response.status().is_success() {
                if let Ok(spot_body) = spot_response.text().await {
                    #[derive(Debug, Deserialize)]
                    struct SpotBalance {
                        currency: String,
                        available: String,
                        locked: String,
                    }

                    if let Ok(balances) = serde_json::from_str::<Vec<SpotBalance>>(&spot_body) {
                        for balance in balances {
                            if balance.currency == "USDT" || balance.currency == "USDC" {
                                let avail = balance.available.parse::<f64>().unwrap_or(0.0);
                                let locked = balance.locked.parse::<f64>().unwrap_or(0.0);
                                let spot_total = avail + locked;
                                
                                if spot_total > 0.01 {
                                    info!("Gate 现货账户 {}: {:.2}", balance.currency, spot_total);
                                    total_balance += spot_total;
                                    available_balance += avail;
                                    margin_used += locked;
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
        let gate_symbol = self.format_symbol(symbol);
        let url_path = format!("/api/v4/futures/{}/contracts/{}", self.settle, gate_symbol);

        let url = format!("{}{}", self.base_url, url_path);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let data: serde_json::Value = serde_json::from_str(&body)?;
        let price: f64 = data["mark_price"]
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

        let gate_symbol = self.format_symbol(symbol);
        let url_path = format!("/api/v4/futures/{}/contracts/{}", self.settle, gate_symbol);

        let url = format!("{}{}", self.base_url, url_path);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let data: serde_json::Value = serde_json::from_str(&body)?;
        
        let rules = TradingRules {
            step_size: data["order_size_min"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            min_qty: data["order_size_min"]
                .as_str()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            quantity_precision: 0,  // Gate 使用整数张数
            price_precision: 2,
        };

        // 缓存规则
        {
            let mut cache = self.rules_cache.write().await;
            cache.insert(symbol.to_string(), rules.clone());
        }

        Ok(rules)
    }

    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()> {
        let url_path = format!("/api/v4/futures/{}/positions/{}/leverage", self.settle, self.format_symbol(symbol));
        let body = json!({
            "leverage": leverage.to_string(),
            "cross_leverage_limit": "0"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            error!("Gate设置杠杆失败: {}", resp_body);
            return Err(anyhow!("Gate设置杠杆失败: {}", resp_body));
        }

        info!("✅ Gate设置杠杆成功: {}x", leverage);
        Ok(())
    }

    async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()> {
        let url_path = format!("/api/v4/futures/{}/positions/{}/margin_mode", self.settle, self.format_symbol(symbol));
        let body = json!({
            "margin_mode": if margin_type == "ISOLATED" { "isolated" } else { "cross" }
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            warn!("Gate设置保证金模式警告: {}", body);
            return Ok(()); // 不阻塞交易
        }

        info!("✅ Gate设置保证金模式成功: {}", margin_type);
        Ok(())
    }

    async fn set_position_mode(&self, dual_side: bool) -> Result<()> {
        let url_path = format!("/api/v4/futures/{}/dual_mode", self.settle);
        let body = json!({
            "dual_mode": dual_side
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            warn!("Gate设置持仓模式警告: {}", body);
            return Ok(()); // 不阻塞交易
        }

        info!(
            "✅ Gate设置持仓模式成功: {}",
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
        let url_path = format!("/api/v4/futures/{}/orders", self.settle);
        let gate_symbol = self.format_symbol(symbol);
        
        let body = json!({
            "contract": gate_symbol,
            "size": quantity as i64,
            "price": "0",
            "tif": "ioc"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            error!("Gate开多失败: {}", resp_body);
            return Err(anyhow!("Gate开多失败: {}", resp_body));
        }

        let resp: serde_json::Value = serde_json::from_str(&resp_body)?;
        info!("✅ Gate开多成功: {} 数量: {}", symbol, quantity);
        
        Ok(OrderResult {
            order_id: resp["id"].as_i64().unwrap_or(0).to_string(),
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
        let url_path = format!("/api/v4/futures/{}/orders", self.settle);
        let gate_symbol = self.format_symbol(symbol);
        
        let body = json!({
            "contract": gate_symbol,
            "size": -(quantity as i64),  // 负数表示做空
            "price": "0",
            "tif": "ioc"
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            error!("Gate开空失败: {}", resp_body);
            return Err(anyhow!("Gate开空失败: {}", resp_body));
        }

        let resp: serde_json::Value = serde_json::from_str(&resp_body)?;
        info!("✅ Gate开空成功: {} 数量: {}", symbol, quantity);
        
        Ok(OrderResult {
            order_id: resp["id"].as_i64().unwrap_or(0).to_string(),
            symbol: symbol.to_string(),
            side: "SELL".to_string(),
            quantity,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }

    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult> {
        let url_path = format!("/api/v4/futures/{}/orders", self.settle);
        let gate_symbol = self.format_symbol(symbol);
        
        // Gate平仓需要用反向数量
        let close_size = if side == "LONG" {
            -(size as i64)
        } else {
            size as i64
        };
        
        let body = json!({
            "contract": gate_symbol,
            "size": close_size,
            "price": "0",
            "tif": "ioc",
            "reduce_only": true
        });
        let body_str = serde_json::to_string(&body)?;

        let headers = self.build_headers("POST", &url_path, "", &body_str);

        let url = format!("{}{}", self.base_url, url_path);
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
            error!("Gate平仓失败: {}", resp_body);
            return Err(anyhow!("Gate平仓失败: {}", resp_body));
        }

        let resp: serde_json::Value = serde_json::from_str(&resp_body)?;
        info!("✅ Gate平仓成功: {} {} {}", symbol, side, size);
        
        Ok(OrderResult {
            order_id: resp["id"].as_i64().unwrap_or(0).to_string(),
            symbol: symbol.to_string(),
            side: if side == "LONG" { "SELL".to_string() } else { "BUY".to_string() },
            quantity: size,
            price: 0.0,
            status: "FILLED".to_string(),
        })
    }
}
