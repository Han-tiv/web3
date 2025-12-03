use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{info, warn};
use tokio::sync::RwLock;

use crate::binance_client::BinanceClient;
use crate::database::{Database, TradeRecord};
use crate::exchange_trait::ExchangeClient;
use crate::staged_position_manager::StagedPosition;
use crate::trading::OrderManager;

/// 持仓追踪信息（Phase 3.1 暂为独立结构）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionTracker {
    pub symbol: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub leverage: u32,
    pub side: String,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub entry_time: DateTime<Utc>,
    pub last_check_time: DateTime<Utc>,
}

/// 持仓操作
#[derive(Debug)]
pub enum PositionAction {
    FullClose {
        symbol: String,
        side: String,
        quantity: f64,
        reason: String,
    },
    PartialClose {
        symbol: String,
        side: String,
        close_quantity: f64,
        close_pct: f64,
        entry_price: f64,
        remaining_quantity: f64,
        stop_loss_order_id: Option<String>,
    },
    Remove(String),
    SetLimitOrder {
        symbol: String,
        side: String,
        quantity: f64,
        limit_price: f64,
        take_profit_order_id: Option<String>,
    },
}

/// 追踪器更新操作
#[derive(Debug)]
pub enum TrackerMutation {
    QuantityAndStopLoss {
        symbol: String,
        new_quantity: f64,
        new_stop_loss_order_id: Option<String>,
    },
    TakeProfitOrder {
        symbol: String,
        new_take_profit_order_id: Option<String>,
    },
}

/// PositionManager：持仓相关基础操作（Phase 3.1）
pub struct PositionManager {
    pub exchange: Arc<BinanceClient>,
    pub order_manager: OrderManager,
    pub db: Database,
}

impl PositionManager {
    pub fn new(exchange: Arc<BinanceClient>, order_manager: OrderManager, db: Database) -> Self {
        Self {
            exchange,
            order_manager,
            db,
        }
    }

    /// 完全平仓
    pub async fn close_position_fully(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        trackers: &Arc<RwLock<HashMap<String, PositionTracker>>>,
        staged_positions: &Arc<RwLock<HashMap<String, StagedPosition>>>,
    ) -> Result<()> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };

        let tracker_snapshot = {
            let trackers = trackers.read().await;
            trackers.get(symbol).cloned()
        };
        let staged_snapshot = {
            let staged = staged_positions.read().await;
            staged.get(symbol).cloned()
        };

        if let Some(tracker) = tracker_snapshot.as_ref() {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                let _ = self.order_manager.cancel_order(symbol, sl_id).await;
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                let _ = self.order_manager.cancel_order(symbol, tp_id).await;
            }
        }

        let current_price = self.exchange.get_current_price(symbol).await?;
        let position_side = if side == "LONG" { "LONG" } else { "SHORT" };
        let limit_price = if side == "LONG" {
            current_price * 0.999
        } else {
            current_price * 1.001
        };
        let order_id = self
            .exchange
            .limit_order(
                symbol,
                quantity,
                close_side,
                limit_price,
                Some(position_side),
                true,
            )
            .await?;
        info!(
            "✅ {} 已完全平仓，限价: {:.4}，订单ID: {}",
            symbol, limit_price, order_id
        );

        self.record_trade_history(
            symbol,
            side,
            quantity,
            limit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await;

        staged_positions.write().await.remove(symbol);

        Ok(())
    }

    /// 部分平仓
    pub async fn close_position_partially(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
    ) -> Result<String> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };
        let current_price = self.exchange.get_current_price(symbol).await?;

        let trading_rules = self.exchange.get_symbol_trading_rules(symbol).await?;
        let min_notional = trading_rules.min_notional.unwrap_or(5.0);
        let notional = quantity * current_price;

        if notional < min_notional {
            warn!(
                "⚠️ {} 部分平仓金额 ${:.2} < ${:.0} (数量: {:.6} × 价格: ${:.2}), 按 reduceOnly 继续执行",
                symbol, notional, min_notional, quantity, current_price
            );
        }

        let position_side = if side == "LONG" { "LONG" } else { "SHORT" };
        let limit_price = if side == "LONG" {
            current_price * 0.999
        } else {
            current_price * 1.001
        };
        let order_id = self
            .exchange
            .limit_order(
                symbol,
                quantity,
                close_side,
                limit_price,
                Some(position_side),
                true,
            )
            .await?;
        info!(
            "✅ {} 已部分平仓下单: {:.6}，限价: {:.4}，订单ID: {}",
            symbol, quantity, limit_price, order_id
        );
        Ok(order_id)
    }

    /// 清理孤立的持仓追踪器
    pub async fn cleanup_orphaned_trackers(
        &self,
        trackers: &Arc<RwLock<HashMap<String, PositionTracker>>>,
    ) {
        let mut trackers = trackers.write().await;
        let mut to_remove = Vec::new();

        for (symbol, tracker) in trackers.iter() {
            match self.exchange.get_positions().await {
                Ok(positions) => {
                    let has_position = positions.iter().any(|p| p.symbol == *symbol);
                    if !has_position {
                        info!("🗑️  清理孤立追踪器: {} (无对应持仓)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
                Err(e) => {
                    warn!("⚠️  获取{}持仓失败(清理检查): {}", symbol, e);
                    warn!("🔍 错误详情: {:?}", e);

                    let age_hours = (Utc::now() - tracker.last_check_time).num_hours();
                    if age_hours >= 24 {
                        warn!("🗑️  清理陈旧追踪器: {} (超过24小时无法验证)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
            }
        }

        for symbol in to_remove {
            trackers.remove(&symbol);
        }

        if !trackers.is_empty() {
            info!("📊 当前持仓追踪器数: {}", trackers.len());
        }
    }

    async fn record_trade_history(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        exit_price: f64,
        tracker_snapshot: Option<PositionTracker>,
        staged_snapshot: Option<StagedPosition>,
    ) {
        let (entry_price, entry_time) = match tracker_snapshot {
            Some(tracker) => (tracker.entry_price, tracker.entry_time),
            None => {
                if let Some(staged) = staged_snapshot {
                    let entry_time = Self::timestamp_ms_to_datetime(staged.trial_entry_time);
                    let entry_price = if staged.avg_cost > 0.0 {
                        staged.avg_cost
                    } else {
                        staged.trial_entry_price
                    };
                    (entry_price, entry_time)
                } else {
                    warn!("⚠️  未找到 {} 的持仓快照，跳过交易记录写入", symbol);
                    return;
                }
            }
        };

        let exit_time = Utc::now();
        let raw_duration = (exit_time - entry_time).num_seconds();
        let hold_duration = if raw_duration < 0 { 0 } else { raw_duration };

        let direction = if side.eq_ignore_ascii_case("LONG") {
            1.0
        } else {
            -1.0
        };
        let pnl = (exit_price - entry_price) * quantity * direction;
        let pnl_pct = if entry_price.abs() <= f64::EPSILON {
            0.0
        } else {
            ((exit_price - entry_price) / entry_price) * 100.0 * direction
        };

        let entry_time_str = entry_time.to_rfc3339();
        let exit_time_str = exit_time.to_rfc3339();
        let trade_record = TradeRecord {
            id: None,
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price,
            exit_price,
            quantity,
            pnl,
            pnl_pct,
            entry_time: entry_time_str,
            exit_time: exit_time_str.clone(),
            hold_duration,
            strategy_tag: None,
            notes: None,
            created_at: Some(exit_time_str),
        };

        if let Err(e) = self.db.insert_trade(&trade_record) {
            warn!("⚠️  记录交易历史失败: {}", e);
        }
    }

    fn timestamp_ms_to_datetime(ms: i64) -> DateTime<Utc> {
        let secs = ms.div_euclid(1000);
        let nsecs = (ms.rem_euclid(1000) as u32) * 1_000_000;
        DateTime::<Utc>::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
    }
}
