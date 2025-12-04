//! Signal Domain Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Option<i64>,
    pub symbol: String,
    pub signal_type: SignalType,
    pub source: SignalSource,
    pub status: SignalStatus,
    pub price: f64,
    pub score: i32,
    pub raw_message: String,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalType {
    /// 资金流入
    FundInflow,
    /// 资金流出
    FundOutflow,
    /// Alpha机会
    Alpha,
    /// FOMO信号
    Fomo,
}

/// 信号来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalSource {
    Telegram,
    Valuescan,
    Internal,
}

/// 信号状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalStatus {
    Pending,
    Processing,
    Processed,
    Rejected,
}

impl Signal {
    /// 创建新信号
    pub fn new(
        symbol: String,
        signal_type: SignalType,
        source: SignalSource,
        raw_message: String,
    ) -> Self {
        Self {
            id: None,
            symbol,
            signal_type,
            source,
            status: SignalStatus::Pending,
            price: 0.0,
            score: 0,
            raw_message,
            created_at: Utc::now(),
            processed_at: None,
        }
    }

    /// 标记为已处理
    pub fn mark_processed(&mut self) {
        self.status = SignalStatus::Processed;
        self.processed_at = Some(Utc::now());
    }

    /// 是否待处理
    pub fn is_pending(&self) -> bool {
        self.status == SignalStatus::Pending
    }
}
