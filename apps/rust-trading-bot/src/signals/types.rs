// 信号类型定义模块
use serde::{Deserialize, Serialize};

/// 信号类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    Inflow,      // 资金流入
    Outflow,     // 资金出逃
    FundEscape,  // 资金出逃（别名）
}

/// 资金异动信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAlert {
    pub coin: String,
    pub raw_message: String,
    pub change_24h: f64,
    pub alert_type: AlertType,
    pub fund_type: String,   // 资金类型描述
    pub price: f64,          // 信号价格
}
