// Hyperliquid 交易所客户端实现
use crate::exchange_trait::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use ethers::prelude::*;
use log::{error, info, warn};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct HyperliquidClient {
    address: String,       // 以太坊地址
    proxy_address: String, // 代理地址（可选）
    secret: String,        // 私钥
    base_url: String,
    rules_cache: Arc<RwLock<HashMap<String, TradingRules>>>,
}

#[derive(Debug, Deserialize)]
struct HyperliquidPosition {
    coin: String,
    #[serde(rename = "szi")]
    size: String,
    #[serde(rename = "entryPx")]
    entry_px: String,
    #[serde(rename = "positionValue")]
    position_value: String,
    #[serde(rename = "unrealizedPnl")]
    unrealized_pnl: String,
    leverage: serde_json::Value,
    #[serde(rename = "marginUsed")]
    margin_used: String,
}

#[derive(Debug, Deserialize)]
struct HyperliquidAccountValue {
    #[serde(rename = "accountValue")]
    account_value: String,
}

#[derive(Debug, Deserialize)]
struct HyperliquidBalance {
    #[serde(rename = "marginSummary")]
    margin_summary: HyperliquidMarginSummary,
}

#[derive(Debug, Deserialize)]
struct HyperliquidMarginSummary {
    #[serde(rename = "accountValue")]
    account_value: String,
    #[serde(rename = "totalMarginUsed")]
    total_margin_used: String,
    #[serde(rename = "totalNtlPos")]
    total_ntl_pos: String,
    #[serde(rename = "totalRawUsd")]
    total_raw_usd: String,
}

#[derive(Debug, Deserialize)]
struct HyperliquidMeta {
    universe: Vec<HyperliquidAssetInfo>,
}

#[derive(Debug, Deserialize)]
struct HyperliquidAssetInfo {
    name: String,
    #[serde(rename = "szDecimals")]
    sz_decimals: i32,
}

impl HyperliquidClient {
    pub fn new(address: String, proxy_address: String, secret: String, testnet: bool) -> Self {
        let base_url = if testnet {
            "https://api.hyperliquid-testnet.xyz".to_string()
        } else {
            "https://api.hyperliquid.xyz".to_string()
        };

        Self {
            address,
            proxy_address,
            secret,
            base_url,
            rules_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用私钥签名消息
    async fn sign_l1_action(&self, action: serde_json::Value) -> Result<Signature> {
        let wallet = self
            .secret
            .parse::<LocalWallet>()
            .map_err(|e| anyhow!("解析私钥失败: {}", e))?;

        // Hyperliquid L1 签名结构
        let connection_id = json!({
            "source": "rust_bot",
            "connectionId": format!("{}{}", wallet.address(), Utc::now().timestamp_millis())
        });

        let message = json!({
            "action": action,
            "nonce": Utc::now().timestamp_millis(),
            "vaultAddress": Value::Null
        });

        let message_str = serde_json::to_string(&message)?;
        let hash = ethers::utils::keccak256(message_str.as_bytes());

        let signature = wallet
            .sign_message(&hash)
            .await
            .map_err(|e| anyhow!("签名失败: {}", e))?;

        Ok(signature)
    }

    /// 发送签名的交易请求
    async fn post_exchange(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/exchange", self.base_url);

        // 将 U256 转换为字节数组
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        signature.r.to_big_endian(&mut r_bytes);
        signature.s.to_big_endian(&mut s_bytes);

        let body = json!({
            "action": action,
            "nonce": nonce,
            "signature": {
                "r": format!("0x{}", hex::encode(r_bytes)),
                "s": format!("0x{}", hex::encode(s_bytes)),
                "v": signature.v
            },
            "vaultAddress": if self.proxy_address.is_empty() { Value::Null } else { json!(self.proxy_address) }
        });

        let client = reqwest::Client::new();
        let response = client.post(&url).json(&body).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            error!("Hyperliquid 交易失败: {}", text);
            return Err(anyhow!("Hyperliquid 交易失败: {}", text));
        }

        let result: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("解析响应失败: {}, 响应: {}", e, text))?;

        Ok(result)
    }

    /// Hyperliquid 使用 POST 请求查询数据
    async fn post_info<T: for<'de> Deserialize<'de>>(
        &self,
        request_type: &str,
        _params: serde_json::Value,
    ) -> Result<T> {
        let url = format!("{}/info", self.base_url);
        let body = json!({
            "type": request_type,
            "user": self.address
        });

        let client = reqwest::Client::new();
        let response = client.post(&url).json(&body).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            error!("Hyperliquid API 错误: {}", text);
            return Err(anyhow!("Hyperliquid API 错误: {}", text));
        }

        serde_json::from_str(&text)
            .map_err(|e| anyhow!("解析 Hyperliquid 响应失败: {}，响应: {}", e, text))
    }

    fn format_symbol(&self, symbol: &str) -> String {
        // Hyperliquid 使用类似 BTC, ETH 的格式（不带 USDT）
        symbol.replace("USDT", "").replace("PERP", "")
    }

    fn unformat_symbol(&self, coin: &str) -> String {
        format!("{}USDT", coin)
    }

    /// 获取资产索引
    async fn get_asset_index(&self, coin: &str) -> Result<u32> {
        let meta: HyperliquidMeta = self.post_info("meta", json!({})).await?;

        meta.universe
            .iter()
            .position(|a| a.name == coin)
            .map(|i| i as u32)
            .ok_or_else(|| anyhow!("未找到资产: {}", coin))
    }
}

#[async_trait]
impl ExchangeClient for HyperliquidClient {
    fn get_exchange_name(&self) -> &str {
        "Hyperliquid"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let response: Vec<HyperliquidPosition> =
            self.post_info("clearinghouseState", json!({})).await?;

        let mut result = Vec::new();
        for pos in response {
            let size: f64 = pos.size.parse().unwrap_or(0.0);
            if size.abs() < 0.0001 {
                continue;
            }

            result.push(Position {
                symbol: self.unformat_symbol(&pos.coin),
                side: if size > 0.0 {
                    "LONG".to_string()
                } else {
                    "SHORT".to_string()
                },
                size: size.abs(),
                entry_price: pos.entry_px.parse().unwrap_or(0.0),
                mark_price: 0.0, // 需要额外查询
                pnl: pos.unrealized_pnl.parse().unwrap_or(0.0),
                leverage: match pos.leverage {
                    serde_json::Value::String(s) => s.parse().unwrap_or(1),
                    serde_json::Value::Number(n) => n.as_i64().unwrap_or(1) as i32,
                    _ => 1,
                },
                margin: pos.margin_used.parse().unwrap_or(0.0),
            });
        }

        Ok(result)
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        // 查询账户状态
        let balance: HyperliquidBalance = self.post_info("clearinghouseState", json!({})).await?;

        let margin_summary = balance.margin_summary;
        let total_balance = margin_summary.account_value.parse::<f64>().unwrap_or(0.0);
        let margin_used = margin_summary
            .total_margin_used
            .parse::<f64>()
            .unwrap_or(0.0);

        info!("Hyperliquid 账户余额: {:.2} USDC", total_balance);

        Ok(AccountInfo {
            total_balance,
            available_balance: total_balance - margin_used,
            unrealized_pnl: 0.0, // 需要从持仓计算
            margin_used,
        })
    }

    async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        let coin = self.format_symbol(symbol);

        #[derive(Debug, Deserialize)]
        struct AllMids {
            mids: HashMap<String, String>,
        }

        let all_mids: AllMids = self.post_info("allMids", json!({})).await?;

        all_mids
            .mids
            .get(&coin)
            .and_then(|price_str| price_str.parse().ok())
            .ok_or_else(|| anyhow!("未找到 {} 的价格", symbol))
    }

    async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules> {
        // 检查缓存
        {
            let cache = self.rules_cache.read().await;
            if let Some(rules) = cache.get(symbol) {
                return Ok(rules.clone());
            }
        }

        let coin = self.format_symbol(symbol);
        let meta: HyperliquidMeta = self.post_info("meta", json!({})).await?;

        let asset_info = meta
            .universe
            .iter()
            .find(|a| a.name == coin)
            .ok_or_else(|| anyhow!("未找到 {} 的交易规则", symbol))?;

        let step_size = 10_f64.powi(-asset_info.sz_decimals);

        let rules = TradingRules {
            step_size,
            min_qty: step_size,
            quantity_precision: asset_info.sz_decimals,
            price_precision: 5, // Hyperliquid 默认价格精度
            tick_size: 0.0001,  // Hyperliquid 行情默认最小价格跳动
        };

        // 缓存规则
        self.rules_cache
            .write()
            .await
            .insert(symbol.to_string(), rules.clone());

        Ok(rules)
    }

    async fn set_leverage(&self, _symbol: &str, _leverage: u32) -> Result<()> {
        // Hyperliquid 通过订单参数设置杠杆，不需要单独设置
        warn!("Hyperliquid 不支持单独设置杠杆，杠杆在订单中指定");
        Ok(())
    }

    async fn set_margin_type(&self, _symbol: &str, _margin_type: &str) -> Result<()> {
        // Hyperliquid 默认全仓模式
        warn!("Hyperliquid 使用全仓模式，无需设置");
        Ok(())
    }

    async fn set_position_mode(&self, _dual_side: bool) -> Result<()> {
        // Hyperliquid 默认单向持仓
        warn!("Hyperliquid 使用单向持仓模式");
        Ok(())
    }

    async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        let coin = self.format_symbol(symbol);
        let price = self.get_current_price(symbol).await?;

        // Hyperliquid 订单格式
        let order = json!({
            "type": "order",
            "orders": [{
                "a": self.get_asset_index(&coin).await?,
                "b": true,  // true = buy (long)
                "p": format!("{:.5}", price * 1.01),  // 稍高于市价确保成交
                "s": format!("{:.8}", quantity),
                "r": false,  // reduce only
                "t": {"limit": {"tif": "Ioc"}}  // Immediate or Cancel
            }],
            "grouping": "na"
        });

        let nonce = Utc::now().timestamp_millis() as u64;
        let signature = self.sign_l1_action(order.clone()).await?;
        let result = self.post_exchange(order, signature, nonce).await?;

        info!("✅ Hyperliquid 开多成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: result["status"]["resting"][0]["oid"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            symbol: symbol.to_string(),
            side: "BUY".to_string(),
            quantity,
            price,
            status: "FILLED".to_string(),
        })
    }

    async fn open_short(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        let coin = self.format_symbol(symbol);
        let price = self.get_current_price(symbol).await?;

        let order = json!({
            "type": "order",
            "orders": [{
                "a": self.get_asset_index(&coin).await?,
                "b": false,  // false = sell (short)
                "p": format!("{:.5}", price * 0.99),  // 稍低于市价确保成交
                "s": format!("{:.8}", quantity),
                "r": false,
                "t": {"limit": {"tif": "Ioc"}}
            }],
            "grouping": "na"
        });

        let nonce = Utc::now().timestamp_millis() as u64;
        let signature = self.sign_l1_action(order.clone()).await?;
        let result = self.post_exchange(order, signature, nonce).await?;

        info!("✅ Hyperliquid 开空成功: {} 数量: {}", symbol, quantity);
        Ok(OrderResult {
            order_id: result["status"]["resting"][0]["oid"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            symbol: symbol.to_string(),
            side: "SELL".to_string(),
            quantity,
            price,
            status: "FILLED".to_string(),
        })
    }

    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult> {
        let coin = self.format_symbol(symbol);
        let price = self.get_current_price(symbol).await?;

        // 平仓是反向操作
        let is_buy = side == "SHORT"; // 平空仓需要买入

        let order = json!({
            "type": "order",
            "orders": [{
                "a": self.get_asset_index(&coin).await?,
                "b": is_buy,
                "p": format!("{:.5}", if is_buy { price * 1.01 } else { price * 0.99 }),
                "s": format!("{:.8}", size),
                "r": true,  // reduce only = true for closing
                "t": {"limit": {"tif": "Ioc"}}
            }],
            "grouping": "na"
        });

        let nonce = Utc::now().timestamp_millis() as u64;
        let signature = self.sign_l1_action(order.clone()).await?;
        let result = self.post_exchange(order, signature, nonce).await?;

        info!(
            "✅ Hyperliquid 平仓成功: {} {} 数量: {}",
            symbol, side, size
        );
        Ok(OrderResult {
            order_id: result["status"]["resting"][0]["oid"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            symbol: symbol.to_string(),
            side: if is_buy { "BUY" } else { "SELL" }.to_string(),
            quantity: size,
            price,
            status: "FILLED".to_string(),
        })
    }
}
