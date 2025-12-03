use crate::deepseek_client::{Kline, TechnicalIndicators};

/// AI 入场分析所需的提示词上下文
#[derive(Debug, Clone)]
pub struct EntryPromptContext<'a> {
    pub symbol: &'a str,
    pub alert_type: &'a str,
    pub alert_message: &'a str,
    pub fund_type: &'a str,
    pub zone_1h_summary: &'a str,
    pub zone_15m_summary: &'a str,
    pub entry_action: &'a str,
    pub entry_reason: &'a str,
    pub klines_5m: &'a [Kline],
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub klines_4h: Option<&'a [Kline]>,
    pub current_price: f64,
    pub change_24h: Option<f64>,
    pub signal_type: Option<&'a str>,
    pub technical_indicators: Option<&'a TechnicalIndicators>,
}

/// AI 持仓管理所需的提示词上下文
#[derive(Debug, Clone)]
pub struct PositionPromptContext<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub entry_price: f64,
    pub current_price: f64,
    pub profit_pct: f64,
    pub hold_duration_hours: f64,
    pub klines_5m: &'a [Kline],
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub indicators: &'a TechnicalIndicators,
    pub support_text: &'a str,
    pub deviation_desc: &'a str,
    pub current_stop_loss: Option<f64>,
    pub current_take_profit: Option<f64>,
}
