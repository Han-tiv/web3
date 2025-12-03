use anyhow::{anyhow, Result};
use chrono::Utc;
use log::{error, info, warn};
use rust_trading_bot::{
    exchange_trait::ExchangeClient,
    staged_position_manager::{StagedPosition, StagedPositionManager},
    trading::OrderManager,
};
use std::sync::Arc;
use teloxide::{prelude::Requester, types::ChatId, Bot as TelegramBot};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    sync::RwLock,
};

use super::super::{
    data::{HistoryRecorder, TrackerManager, TradeRecordParams},
    modules::types::PositionTracker,
};

pub struct PositionCloser {
    exchange: Arc<dyn ExchangeClient + Send + Sync>,
    order_manager: Arc<OrderManager>,
    history_recorder: Arc<HistoryRecorder>,
    tracker_manager: Arc<TrackerManager>,
    staged_manager: Arc<RwLock<StagedPositionManager>>,
    telegram_bot: Option<Arc<TelegramBot>>,
}

impl PositionCloser {
    pub fn new(
        exchange: Arc<dyn ExchangeClient + Send + Sync>,
        order_manager: Arc<OrderManager>,
        history_recorder: Arc<HistoryRecorder>,
        tracker_manager: Arc<TrackerManager>,
        staged_manager: Arc<RwLock<StagedPositionManager>>,
        telegram_bot: Option<Arc<TelegramBot>>,
    ) -> Self {
        Self {
            exchange,
            order_manager,
            history_recorder,
            tracker_manager,
            staged_manager,
            telegram_bot,
        }
    }

    /// 完全平仓
    pub async fn close_fully(&self, params: CloseParams) -> Result<()> {
        let symbol = params.symbol.as_str();
        info!("🔄 准备全仓平仓: {}", symbol);

        let tracker_snapshot = self.tracker_manager.get_tracker(symbol);
        let staged_snapshot = {
            let staged = self.staged_manager.read().await;
            staged.positions.get(symbol).cloned()
        };

        let positions = self.exchange.get_positions().await?;
        let real_position = positions.into_iter().find(|p| p.symbol == symbol);
        let (real_size, side) = match real_position {
            Some(pos) => {
                if pos.size.abs() < 0.0001 {
                    warn!("⚠️  {} 实际持仓过小 ({:.8}),清理追踪记录", symbol, pos.size);
                    self.clear_tracking(symbol).await;
                    return Ok(());
                }
                (pos.size.abs(), pos.side.to_ascii_uppercase())
            }
            None => {
                warn!("⚠️  {} 无持仓,清理追踪记录", symbol);
                self.clear_tracking(symbol).await;
                return Ok(());
            }
        };

        info!("📊 {} 实时持仓: {:.8} ({})", symbol, real_size, side);

        if let Some(tracker) = tracker_snapshot.as_ref() {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, sl_id).await {
                    warn!("⚠️  取消 {} 止损单失败: {}", symbol, e);
                }
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, tp_id).await {
                    warn!("⚠️  取消 {} 止盈单失败: {}", symbol, e);
                }
            }
        }

        let exit_price = self.exchange.get_current_price(symbol).await?;

        self.exchange
            .close_position(symbol, &side, real_size)
            .await?;
        info!("✅ {} 全仓平仓成功", symbol);

        self.finalize_close(
            symbol,
            &side,
            real_size,
            exit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await
    }

    /// 带重试的完全平仓，失败时指数退避并最终触发市价单兜底
    pub async fn close_fully_with_retry(&self, params: CloseParams) -> Result<()> {
        let symbol = params.symbol.clone();
        let retries = params.max_retries.max(1);

        for attempt in 1..=retries {
            match self
                .close_fully(CloseParams {
                    symbol: symbol.clone(),
                    ..params.clone()
                })
                .await
            {
                Ok(_) => {
                    info!("✅ {} 平仓成功 (尝试 {}/{})", symbol, attempt, retries);
                    return Ok(());
                }
                Err(e) => {
                    if attempt < retries {
                        let backoff_secs = 2_u64.pow(attempt);
                        warn!(
                            "⚠️  {} 平仓失败 (尝试 {}/{}): {} - {}秒后重试",
                            symbol, attempt, retries, e, backoff_secs
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                    } else {
                        error!("❌ {} 多次重试后仍失败: {}", symbol, e);
                        warn!("🔄 最后尝试: 使用市价单强制平仓 {}", symbol);
                        match self.try_market_fallback(params.clone()).await {
                            Ok(_) => {
                                info!("✅ 市价单 fallback 成功: {}", symbol);
                                return Ok(());
                            }
                            Err(fallback_err) => {
                                error!("❌ 市价单 fallback 也失败: {}", fallback_err);
                                let alert_msg = format!(
                                    "平仓完全失败 - 限价单: {} / 市价单: {}",
                                    e, fallback_err
                                );
                                let _ = self.send_alert(&symbol, &alert_msg).await;
                                return Err(anyhow!(alert_msg));
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow!("不应到达此处"))
    }

    /// 使用实时仓位信息执行部分平仓，返回剩余仓位
    pub async fn close_partially(&self, params: PartialCloseParams) -> Result<f64> {
        if params.close_pct <= 0.0 {
            return Err(anyhow!(
                "{} 部分平仓百分比无效: {}",
                params.symbol,
                params.close_pct
            ));
        }

        info!("📉 准备部分平仓: {} ({}%)", params.symbol, params.close_pct);
        let positions = self.exchange.get_positions().await?;
        let real_position = positions
            .iter()
            .find(|p| p.symbol == params.symbol)
            .ok_or_else(|| anyhow!("{} 无实时持仓", params.symbol))?;
        let real_size = real_position.size.abs();

        if real_size <= f64::EPSILON {
            warn!("⚠️  {} 实际持仓数量为零, 直接清理追踪器", params.symbol);
            self.clear_tracking(&params.symbol).await;
            return Ok(0.0);
        }

        let side = if real_position.size > 0.0 {
            "LONG"
        } else {
            "SHORT"
        };
        let pct = params.close_pct.min(100.0);
        let mut close_amount = real_size * (pct / 100.0);
        if close_amount <= f64::EPSILON {
            return Err(anyhow!(
                "{} 计算部分平仓数量过小: {:.8}",
                params.symbol,
                close_amount
            ));
        }

        close_amount = close_amount.min(real_size);
        info!(
            "📊 {} 实时持仓: {:.8}, 平仓 {}% -> {:.8}",
            params.symbol, real_size, pct, close_amount
        );

        if close_amount / real_size > 0.9999 {
            info!(
                "⚠️  {} 计划部分平仓接近全仓，建议直接调用全平逻辑",
                params.symbol
            );
        }

        if let Err(e) = self
            .exchange
            .close_position(&params.symbol, side, close_amount)
            .await
        {
            error!("❌ {} 部分平仓失败: {}", params.symbol, e);
            if let Ok(updated_positions) = self.exchange.get_positions().await {
                if let Some(updated_pos) =
                    updated_positions.iter().find(|p| p.symbol == params.symbol)
                {
                    let trackers = self.tracker_manager.shared();
                    let mut writer = trackers.write().await;
                    if let Some(tracker) = writer.get_mut(&params.symbol) {
                        tracker.quantity = updated_pos.size.abs();
                        tracker.last_check_time = Utc::now();
                        warn!(
                            "⚠️  平仓失败但已同步 tracker: {} = {:.8}",
                            params.symbol, tracker.quantity
                        );
                    }
                } else {
                    self.clear_tracking(&params.symbol).await;
                    warn!("⚠️  {} 持仓已消失,清理 tracker", params.symbol);
                }
            }
            return Err(e);
        }

        info!("✅ {} 部分平仓成功: {:.8}", params.symbol, close_amount);
        let updated_positions = self.exchange.get_positions().await?;
        let remaining_quantity = updated_positions
            .iter()
            .find(|p| p.symbol == params.symbol)
            .map(|p| p.size.abs())
            .unwrap_or(0.0);

        let trackers = self.tracker_manager.shared();
        let mut writer = trackers.write().await;
        if remaining_quantity <= 0.0001 {
            writer.remove(&params.symbol);
            info!("🗑️  {} 部分平仓后无剩余持仓, 已清理 tracker", params.symbol);
        } else if let Some(tracker) = writer.get_mut(&params.symbol) {
            tracker.quantity = remaining_quantity;
            tracker.last_check_time = Utc::now();
            info!(
                "📝 更新 tracker: {} 剩余 {:.8}",
                params.symbol, tracker.quantity
            );
        }

        Ok(remaining_quantity.max(0.0))
    }

    pub async fn finalize_close(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        exit_price: f64,
        tracker_snapshot: Option<PositionTracker>,
        staged_snapshot: Option<StagedPosition>,
    ) -> Result<()> {
        let record = TradeRecordParams::from_snapshots(
            symbol.to_string(),
            side.to_string(),
            exit_price,
            quantity,
            tracker_snapshot.clone(),
            staged_snapshot.clone(),
        );
        self.history_recorder.record_trade(record).await?;
        self.clear_tracking(symbol).await;
        Ok(())
    }

    async fn try_market_fallback(&self, params: CloseParams) -> Result<()> {
        let symbol = params.symbol.as_str();
        warn!("🔄 启动市价单 fallback 强制平仓: {}", symbol);

        let tracker_snapshot = self.tracker_manager.get_tracker(symbol);
        let staged_snapshot = {
            let staged = self.staged_manager.read().await;
            staged.positions.get(symbol).cloned()
        };

        if let Some(tracker) = tracker_snapshot.as_ref() {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, sl_id).await {
                    warn!("⚠️  Fallback 取消止损单失败: {}", e);
                }
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, tp_id).await {
                    warn!("⚠️  Fallback 取消止盈单失败: {}", e);
                }
            }
        }

        let positions = match self.exchange.get_positions().await {
            Ok(data) => data,
            Err(fetch_err) => {
                return Err(fetch_err);
            }
        };

        let Some(pos) = positions.iter().find(|p| p.symbol == symbol) else {
            warn!("⚠️  市价单 Fallback 未找到 {} 持仓,自动清理追踪", symbol);
            self.clear_tracking(symbol).await;
            return Ok(());
        };

        let fallback_side = pos.side.to_ascii_uppercase();
        let fallback_size = pos.size.abs();
        if fallback_size <= 0.0 {
            warn!(
                "⚠️  市价单 Fallback 检测到 {} 仓位数量为0，直接清理追踪记录",
                symbol
            );
            self.clear_tracking(symbol).await;
            return Ok(());
        }

        let exit_price = match self.exchange.get_current_price(symbol).await {
            Ok(price) => price,
            Err(price_err) => {
                warn!(
                    "⚠️  获取 {} 最新价格失败 ({})，使用标记价 {:.4}",
                    symbol, price_err, pos.mark_price
                );
                pos.mark_price
            }
        };

        self.exchange
            .close_position(symbol, &fallback_side, fallback_size)
            .await?;

        info!("✅ 市价单 Fallback 平仓成功: {}", symbol);
        self.finalize_close(
            symbol,
            &fallback_side,
            fallback_size,
            exit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await
    }

    pub async fn send_alert(&self, symbol: &str, reason: &str) -> Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let alert_msg = format!(
            "🚨 紧急告警 - 需人工干预\n\n交易对: {}\n时间: {}\n原因: {}\n\n请立即检查持仓状态!",
            symbol, timestamp, reason
        );

        error!("🚨 CRITICAL ALERT [{}] {}", symbol, reason);
        error!("{}", alert_msg);

        if let Some(bot) = self.telegram_bot.as_ref() {
            match std::env::var("TELEGRAM_ALERT_CHAT_ID") {
                Ok(chat_id) => match chat_id.parse::<i64>() {
                    Ok(chat_id_i64) => {
                        let chat = ChatId(chat_id_i64);
                        if let Err(e) = bot.send_message(chat, &alert_msg).await {
                            error!("❌ Telegram 告警发送失败: {}", e);
                        } else {
                            info!("✅ Telegram 告警已发送");
                        }
                    }
                    Err(e) => warn!("⚠️ TELEGRAM_ALERT_CHAT_ID 解析失败: {}", e),
                },
                Err(_) => warn!("⚠️ 未配置 TELEGRAM_ALERT_CHAT_ID, Telegram 告警不可用"),
            }
        }

        if let Err(e) = fs::create_dir_all("logs").await {
            error!("❌ 创建日志目录失败: {}", e);
        }

        let alert_file = "logs/critical_alerts.log";
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(alert_file)
            .await
        {
            Ok(mut file) => {
                let log_entry = format!("[{}] {}\n{}\n\n", timestamp, symbol, reason);
                if let Err(e) = file.write_all(log_entry.as_bytes()).await {
                    error!("❌ 写入告警日志失败: {}", e);
                }
            }
            Err(e) => error!("❌ 打开告警日志失败: {}", e),
        }

        Ok(())
    }

    async fn clear_tracking(&self, symbol: &str) {
        self.tracker_manager.clear_tracker(symbol);
        let mut staged_manager = self.staged_manager.write().await;
        staged_manager.positions.remove(symbol);
    }
}

#[derive(Clone)]
pub struct CloseParams {
    pub symbol: String,
    pub max_retries: u32,
    pub reason: Option<String>,
}

impl CloseParams {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            max_retries: 3,
            reason: None,
        }
    }
}

#[derive(Clone)]
pub struct PartialCloseParams {
    pub symbol: String,
    pub close_pct: f64,
}

impl PartialCloseParams {
    pub fn new(symbol: impl Into<String>, close_pct: f64) -> Self {
        Self {
            symbol: symbol.into(),
            close_pct,
        }
    }
}
