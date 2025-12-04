use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 资金预警信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAlert {
    pub coin: String,
    pub alert_type: AlertType,
    pub price: f64,
    pub change_24h: f64,
    pub fund_type: String,
    pub timestamp: DateTime<Utc>,
    pub raw_message: String,
}

/// 预警类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)] // AlphaOpportunity 和 FomoSignal 暂保留
pub enum AlertType {
    AlphaOpportunity,
    FomoSignal,
    FundInflow,
    FundEscape,
}
