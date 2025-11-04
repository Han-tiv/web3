// Solana 钱包客户端（使用 HTTP RPC）
use crate::exchange_trait::*;
use crate::price_service::PriceService;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct SolanaWallet {
    address: String,
    private_key: String,
    rpc_url: String,
    price_service: Arc<PriceService>,
}

impl SolanaWallet {
    pub fn new(address: String, private_key: String, testnet: bool) -> Self {
        let rpc_url = if testnet {
            "https://api.devnet.solana.com".to_string()
        } else {
            "https://api.mainnet-beta.solana.com".to_string()
        };

        Self {
            address,
            private_key,
            rpc_url,
            price_service: Arc::new(PriceService::new()),
        }
    }

    /// 获取 SOL 余额（使用 HTTP RPC）
    async fn get_sol_balance(&self) -> Result<f64> {
        #[derive(Deserialize)]
        struct RpcResponse {
            result: RpcResult,
        }

        #[derive(Deserialize)]
        struct RpcResult {
            value: u64,
        }

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [self.address]
        });

        let client = reqwest::Client::new();
        let response = client.post(&self.rpc_url).json(&body).send().await?;

        let rpc_response: RpcResponse = response.json().await?;

        // SOL has 9 decimals
        let balance_sol = rpc_response.result.value as f64 / 1_000_000_000.0;

        Ok(balance_sol)
    }

    // Note: SPL Token 查询需要额外依赖，暂不实现
    // 如需查询 USDC/USDT 等 SPL Token，请使用 Solana 浏览器或钱包
}

#[async_trait]
impl ExchangeClient for SolanaWallet {
    fn get_exchange_name(&self) -> &str {
        "Solana"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        // 链上钱包没有持仓概念
        Ok(Vec::new())
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let mut total_balance = 0.0;

        // 查询 SOL 余额
        match self.get_sol_balance().await {
            Ok(sol) => {
                if sol > 0.001 {
                    info!("Solana SOL 余额: {:.4}", sol);

                    // 获取 SOL 实时价格
                    match self.price_service.get_price("sol").await {
                        Ok(price) => {
                            let value = sol * price;
                            info!("Solana SOL 价值: ${:.2} (@ ${:.2}/SOL)", value, price);
                            total_balance += value;
                        }
                        Err(e) => {
                            warn!("获取 SOL 价格失败: {}，使用默认价格 $150", e);
                            total_balance += sol * 150.0;
                        }
                    }
                }
            }
            Err(e) => warn!("查询 SOL 余额失败: {}", e),
        }

        Ok(AccountInfo {
            total_balance,
            available_balance: total_balance,
            unrealized_pnl: 0.0,
            margin_used: 0.0,
        })
    }

    async fn get_current_price(&self, _symbol: &str) -> Result<f64> {
        Err(anyhow!("Solana 钱包不支持价格查询"))
    }

    async fn get_symbol_trading_rules(&self, _symbol: &str) -> Result<TradingRules> {
        Err(anyhow!("Solana 钱包不支持交易规则查询"))
    }

    async fn set_leverage(&self, _symbol: &str, _leverage: u32) -> Result<()> {
        Ok(()) // 链上钱包无杠杆概念
    }

    async fn set_margin_type(&self, _symbol: &str, _margin_type: &str) -> Result<()> {
        Ok(()) // 链上钱包无保证金类型
    }

    async fn set_position_mode(&self, _dual_side: bool) -> Result<()> {
        Ok(()) // 链上钱包无持仓模式
    }

    async fn open_long(
        &self,
        _symbol: &str,
        _quantity: f64,
        _leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        Err(anyhow!("Solana 钱包不支持合约交易，请使用 DEX"))
    }

    async fn open_short(
        &self,
        _symbol: &str,
        _quantity: f64,
        _leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        Err(anyhow!("Solana 钱包不支持合约交易，请使用 DEX"))
    }

    async fn close_position(&self, _symbol: &str, _side: &str, _size: f64) -> Result<OrderResult> {
        Err(anyhow!("Solana 钱包不支持合约交易"))
    }
}
