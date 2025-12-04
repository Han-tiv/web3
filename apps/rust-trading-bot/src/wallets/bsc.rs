// BSC (Binance Smart Chain) 钱包客户端
use crate::exchange_trait::*;
use crate::price_service::PriceService;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use log::{info, warn};
use std::sync::Arc;

#[derive(Clone)]
pub struct BscWallet {
    address: String,
    #[allow(dead_code)]
    private_key: String,
    rpc_url: String,
    #[allow(dead_code)]
    chain_id: u64,
    price_service: Arc<PriceService>,
}

impl BscWallet {
    pub fn new(address: String, private_key: String, testnet: bool) -> Self {
        let (rpc_url, chain_id) = if testnet {
            (
                "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
                97,
            )
        } else {
            ("https://bsc-dataseed1.binance.org".to_string(), 56)
        };

        Self {
            address,
            private_key,
            rpc_url,
            chain_id,
            price_service: Arc::new(PriceService::new()),
        }
    }

    /// 获取 BNB 余额
    async fn get_bnb_balance(&self) -> Result<f64> {
        let provider = Provider::<Http>::try_from(&self.rpc_url)?;
        let address: Address = self.address.parse()?;

        let balance = provider.get_balance(address, None).await?;
        let balance_bnb = ethers::utils::format_units(balance, "ether")?;

        Ok(balance_bnb.parse()?)
    }

    /// 获取 USDT (BEP20) 余额
    async fn get_usdt_balance(&self) -> Result<f64> {
        // BSC USDT 合约地址
        let usdt_address: Address = "0x55d398326f99059fF775485246999027B3197955".parse()?;
        let provider = Arc::new(Provider::<Http>::try_from(&self.rpc_url)?);

        // ERC20 balanceOf ABI
        abigen!(
            ERC20,
            r#"[{"constant":true,"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"type":"function"}]"#
        );

        let contract = ERC20::new(usdt_address, provider);
        let address: Address = self.address.parse()?;
        let balance: U256 = contract.balance_of(address).call().await?;

        // USDT has 18 decimals on BSC
        let balance_usdt = ethers::utils::format_units(balance, 18)?;

        Ok(balance_usdt.parse()?)
    }

    /// 获取 BUSD 余额
    async fn get_busd_balance(&self) -> Result<f64> {
        let busd_address: Address = "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".parse()?;
        let provider = Arc::new(Provider::<Http>::try_from(&self.rpc_url)?);

        abigen!(
            ERC20Token,
            r#"[{"constant":true,"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"type":"function"}]"#
        );

        let contract = ERC20Token::new(busd_address, provider);
        let address: Address = self.address.parse()?;
        let balance: U256 = contract.balance_of(address).call().await?;

        let balance_busd = ethers::utils::format_units(balance, 18)?;

        Ok(balance_busd.parse()?)
    }
}

#[async_trait]
impl ExchangeClient for BscWallet {
    fn get_exchange_name(&self) -> &str {
        "BSC"
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        // 链上钱包没有持仓概念
        Ok(Vec::new())
    }

    async fn get_account_info(&self) -> Result<AccountInfo> {
        let mut total_balance = 0.0;

        // 查询 BNB 余额
        match self.get_bnb_balance().await {
            Ok(bnb) => {
                if bnb > 0.001 {
                    info!("BSC BNB 余额: {:.4}", bnb);

                    // 获取 BNB 实时价格
                    match self.price_service.get_price("bnb").await {
                        Ok(price) => {
                            let value = bnb * price;
                            info!("BSC BNB 价值: ${:.2} (@ ${:.2}/BNB)", value, price);
                            total_balance += value;
                        }
                        Err(e) => {
                            warn!("获取 BNB 价格失败: {}，使用默认价格 $600", e);
                            total_balance += bnb * 600.0;
                        }
                    }
                }
            }
            Err(e) => warn!("查询 BNB 余额失败: {}", e),
        }

        // 查询 USDT 余额
        match self.get_usdt_balance().await {
            Ok(usdt) => {
                if usdt > 0.01 {
                    info!("BSC USDT 余额: {:.2}", usdt);
                    total_balance += usdt;
                }
            }
            Err(e) => warn!("查询 USDT 余额失败: {}", e),
        }

        // 查询 BUSD 余额
        match self.get_busd_balance().await {
            Ok(busd) => {
                if busd > 0.01 {
                    info!("BSC BUSD 余额: {:.2}", busd);
                    total_balance += busd;
                }
            }
            Err(e) => warn!("查询 BUSD 余额失败: {}", e),
        }

        Ok(AccountInfo {
            total_balance,
            available_balance: total_balance,
            unrealized_pnl: 0.0,
            margin_used: 0.0,
        })
    }

    async fn get_current_price(&self, _symbol: &str) -> Result<f64> {
        Err(anyhow!("BSC 钱包不支持价格查询"))
    }

    async fn get_symbol_trading_rules(&self, _symbol: &str) -> Result<TradingRules> {
        Err(anyhow!("BSC 钱包不支持交易规则查询"))
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
        Err(anyhow!("BSC 钱包不支持合约交易，请使用 DEX"))
    }

    async fn open_short(
        &self,
        _symbol: &str,
        _quantity: f64,
        _leverage: u32,
        _margin_type: &str,
        _dual_side: bool,
    ) -> Result<OrderResult> {
        Err(anyhow!("BSC 钱包不支持合约交易，请使用 DEX"))
    }

    async fn close_position(&self, _symbol: &str, _side: &str, _size: f64) -> Result<OrderResult> {
        Err(anyhow!("BSC 钱包不支持合约交易"))
    }
}
