use anyhow::Result;
use chrono::{DateTime, Utc};
use log::warn;
use rust_trading_bot::{
    config::database::{Database, TradeRecord as DbTradeRecord},
    staged_position_manager::StagedPosition,
};
use std::sync::Arc;

use super::super::{modules::types::PositionTracker, utils::timestamp_ms_to_datetime};

/// 交易历史记录器
///
/// 负责统一写入数据库记录，避免在核心交易流程中散落重复逻辑。
pub struct HistoryRecorder {
    db: Arc<Database>,
}

impl HistoryRecorder {
    /// 创建新的记录器实例。
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 记录一笔平仓交易；若缺少必要上下文则会安全跳过。
    pub async fn record_trade(&self, params: TradeRecordParams) -> Result<()> {
        let (entry_price, entry_time) = match Self::resolve_entry_context(&params) {
            Some(data) => data,
            None => {
                warn!("⚠️  未找到 {} 的持仓快照，跳过交易记录写入", params.symbol);
                return Ok(());
            }
        };

        let exit_time = params.exit_time.unwrap_or_else(Utc::now);
        let raw_duration = (exit_time - entry_time).num_seconds();
        let hold_duration = raw_duration.max(0);

        let direction = if params.side.eq_ignore_ascii_case("LONG") {
            1.0
        } else {
            -1.0
        };
        let computed_pnl = (params.exit_price - entry_price) * params.quantity * direction;
        let pnl = params.pnl.unwrap_or(computed_pnl);
        let pnl_pct = if entry_price.abs() <= f64::EPSILON {
            0.0
        } else {
            ((params.exit_price - entry_price) / entry_price) * 100.0 * direction
        };

        let entry_time_str = entry_time.to_rfc3339();
        let exit_time_str = exit_time.to_rfc3339();
        let trade_record = DbTradeRecord {
            id: None,
            symbol: params.symbol.clone(),
            side: params.side.clone(),
            entry_price,
            exit_price: params.exit_price,
            quantity: params.quantity,
            pnl,
            pnl_pct,
            entry_time: entry_time_str,
            exit_time: exit_time_str.clone(),
            hold_duration,
            strategy_tag: params.strategy_tag.clone(),
            notes: params.reason.clone(),
            created_at: Some(exit_time_str),
        };

        if let Err(e) = self.db.insert_trade(&trade_record) {
            warn!("⚠️  记录交易历史失败: {}", e);
        }

        Ok(())
    }

    fn resolve_entry_context(params: &TradeRecordParams) -> Option<(f64, DateTime<Utc>)> {
        if let (Some(price), Some(time)) = (params.entry_price, params.entry_time) {
            return Some((price, time));
        }

        if let Some(tracker) = &params.tracker_snapshot {
            return Some((tracker.entry_price, tracker.entry_time));
        }

        if let Some(staged) = &params.staged_snapshot {
            let entry_time = timestamp_ms_to_datetime(staged.trial_entry_time);
            let entry_price = if staged.avg_cost > 0.0 {
                staged.avg_cost
            } else {
                staged.trial_entry_price
            };
            return Some((entry_price, entry_time));
        }

        None
    }
}

/// 写入交易历史所需的参数集合。
#[derive(Debug, Clone)]
pub struct TradeRecordParams {
    pub symbol: String,
    pub side: String,
    pub entry_price: Option<f64>,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub reason: Option<String>,
    pub entry_time: Option<DateTime<Utc>>,
    pub exit_time: Option<DateTime<Utc>>,
    pub strategy_tag: Option<String>,
    pub tracker_snapshot: Option<PositionTracker>,
    pub staged_snapshot: Option<StagedPosition>,
}

impl TradeRecordParams {
    /// 辅助构造函数，方便最常见的平仓记录场景。
    pub fn from_snapshots(
        symbol: String,
        side: String,
        exit_price: f64,
        quantity: f64,
        tracker_snapshot: Option<PositionTracker>,
        staged_snapshot: Option<StagedPosition>,
    ) -> Self {
        Self {
            symbol,
            side,
            entry_price: None,
            exit_price,
            quantity,
            pnl: None,
            reason: None,
            entry_time: None,
            exit_time: None,
            strategy_tag: None,
            tracker_snapshot,
            staged_snapshot,
        }
    }
}
