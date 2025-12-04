//! Order Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Option<i64>,
    pub order_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub price: f64,
    pub quantity: f64,
    pub created_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
}

/// 订单类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Expired,
}

impl Order {
    /// 是否已成交
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// 是否活跃
    pub fn is_active(&self) -> bool {
        self.status == OrderStatus::Pending
    }
}
