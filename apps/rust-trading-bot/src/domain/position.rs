//! Position Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Option<i64>,
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub leverage: u32,
    pub status: PositionStatus,
    pub pnl: f64,
    pub pnl_percentage: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// 持仓方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
}

/// 持仓状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionStatus {
    Active,
    Closing,
    Closed,
}

impl Position {
    /// 计算PnL
    pub fn calculate_pnl(&mut self) {
        match self.side {
            PositionSide::Long => {
                self.pnl = (self.current_price - self.entry_price) * self.quantity;
            }
            PositionSide::Short => {
                self.pnl = (self.entry_price - self.current_price) * self.quantity;
            }
        }

        if self.entry_price > 0.0 {
            self.pnl_percentage =
                (self.pnl / (self.entry_price * self.quantity)) * 100.0 * self.leverage as f64;
        }
    }

    /// 是否为活跃持仓
    pub fn is_active(&self) -> bool {
        self.status == PositionStatus::Active
    }
}
