use crate::exchanges::binance::BinanceClient;
use crate::exchange_trait::{ExchangeClient, Position};
use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct CopyTradeConfig {
    pub copy_ratio: f64,          // 跟单比例
    pub max_position_size: f64,   // 最大仓位 USDT
    pub leverage: u32,            // 杠杆倍数
    pub enable_stop_loss: bool,   // 是否启用止损
    pub stop_loss_percent: f64,   // 止损百分比
    pub fixed_margin_usdt: f64,   // 固定保证金（通过env配置，默认2 USDT）
    pub margin_type: String,      // 逐仓/全仓 ("ISOLATED"/"CROSSED")
    pub dual_side_position: bool, // 持仓模式：true=双向，false=单向
}

pub struct CopyTrader {
    leader_client: Arc<BinanceClient>,
    follower_client: Arc<BinanceClient>,
    config: CopyTradeConfig,
    last_positions: Arc<Mutex<Vec<Position>>>,
}

impl CopyTrader {
    pub fn new(
        leader_client: BinanceClient,
        follower_client: BinanceClient,
        config: CopyTradeConfig,
    ) -> Self {
        Self {
            leader_client: Arc::new(leader_client),
            follower_client: Arc::new(follower_client),
            config,
            last_positions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 启动跟单监控
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("🚀 开始监控带单者持仓变化...");
        info!("📊 跟单比例: {}%", self.config.copy_ratio * 100.0);
        info!("💰 最大仓位: {} USDT", self.config.max_position_size);
        info!("⚡ 杠杆倍数: {}x", self.config.leverage);

        let mut check_interval = interval(Duration::from_secs(5));

        loop {
            check_interval.tick().await;

            if let Err(e) = self.check_and_copy().await {
                error!("❌ 跟单检查失败: {}", e);
            }
        }
    }

    /// 检查并执行跟单
    async fn check_and_copy(&self) -> Result<()> {
        // 获取带单者当前持仓
        let current_positions = self.leader_client.get_positions().await?;

        // 获取上次持仓记录
        let mut last_positions = self.last_positions.lock().await;

        // 检测新开仓
        for pos in &current_positions {
            if !last_positions.iter().any(|p| p.symbol == pos.symbol) {
                info!("🆕 检测到新开仓: {} {} {}", pos.symbol, pos.side, pos.size);
                self.copy_open_position(pos).await?;
            }
        }

        // 检测平仓
        for old_pos in last_positions.iter() {
            if !current_positions.iter().any(|p| p.symbol == old_pos.symbol) {
                info!("📤 检测到平仓: {} {}", old_pos.symbol, old_pos.side);
                self.copy_close_position(old_pos).await?;
            }
        }

        // 更新持仓记录
        *last_positions = current_positions;

        Ok(())
    }

    /// 跟单开仓
    async fn copy_open_position(&self, leader_pos: &Position) -> Result<()> {
        // 使用固定保证金与杠杆，结合交易规则按步长对齐计算数量
        let price = self
            .follower_client
            .get_current_price(&leader_pos.symbol)
            .await?;
        let rules = self
            .follower_client
            .get_symbol_trading_rules(&leader_pos.symbol)
            .await?;
        let desired_margin = self
            .config
            .fixed_margin_usdt
            .min(self.config.max_position_size);
        let copy_quantity = self.follower_client.calculate_quantity_with_margin(
            price,
            desired_margin,
            self.config.leverage,
            &rules,
        )?;

        info!(
            "💼 跟单开仓: {} {} x{} 杠杆, 数量: {:.4}",
            leader_pos.symbol, leader_pos.side, self.config.leverage, copy_quantity
        );

        // 执行开仓
        match leader_pos.side.as_str() {
            "LONG" => {
                self.follower_client
                    .open_long(
                        &leader_pos.symbol,
                        copy_quantity,
                        self.config.leverage,
                        &self.config.margin_type,
                        self.config.dual_side_position,
                    )
                    .await?;
            }
            "SHORT" => {
                self.follower_client
                    .open_short(
                        &leader_pos.symbol,
                        copy_quantity,
                        self.config.leverage,
                        &self.config.margin_type,
                        self.config.dual_side_position,
                    )
                    .await?;
            }
            _ => warn!("⚠️ 未知持仓方向: {}", leader_pos.side),
        }

        // 设置止损（暂时禁用）
        /*
        if self.config.enable_stop_loss {
            let stop_price = self.calculate_stop_loss_price(leader_pos);
            self.follower_client
                .set_stop_loss(
                    &leader_pos.symbol,
                    &leader_pos.side,
                    copy_quantity,
                    stop_price,
                    None,
                )
                .await?;
        }
        */

        Ok(())
    }

    /// 跟单平仓
    async fn copy_close_position(&self, leader_pos: &Position) -> Result<()> {
        // 获取自己的持仓
        let my_positions = self.follower_client.get_positions().await?;

        if let Some(my_pos) = my_positions.iter().find(|p| p.symbol == leader_pos.symbol) {
            info!(
                "💵 跟单平仓: {} {} 数量: {:.4}, 盈亏: {:.2} USDT",
                my_pos.symbol, my_pos.side, my_pos.size, my_pos.pnl
            );

            self.follower_client
                .close_position(&my_pos.symbol, &my_pos.side, my_pos.size)
                .await?;
        } else {
            warn!("⚠️ 未找到对应持仓: {}", leader_pos.symbol);
        }

        Ok(())
    }

    /// 获取跟单统计
    pub async fn get_statistics(&self) -> Result<CopyTradeStats> {
        let follower_account = self.follower_client.get_account_info().await?;
        let positions = self.follower_client.get_positions().await?;

        let total_pnl: f64 = positions.iter().map(|p| p.pnl).sum();

        Ok(CopyTradeStats {
            balance: follower_account.totalWalletBalance.parse()?,
            available_balance: follower_account.availableBalance.parse()?,
            total_pnl,
            position_count: positions.len(),
            positions,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CopyTradeStats {
    pub balance: f64,
    pub available_balance: f64,
    pub total_pnl: f64,
    pub position_count: usize,
    pub positions: Vec<Position>,
}
