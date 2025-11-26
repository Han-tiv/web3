// V2 数据结构 - Valuescan方法论
// 基于新的评分系统和关键位交易法

use serde::{Deserialize, Deserializer, Serialize};

/// 默认币种类型
fn default_coin_type() -> String {
    "altcoin".to_string()
}

/// 自定义反序列化：将 null/string 转换为默认 f64
fn deserialize_flexible_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct FlexibleF64Visitor;

    impl<'de> Visitor<'de> for FlexibleF64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number, string representation of number, or null")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // 尝试解析数字，失败则返回0.0
            value.parse::<f64>().ok().unwrap_or(0.0).pipe(Ok)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(0.0)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(0.0)
        }
    }

    deserializer.deserialize_any(FlexibleF64Visitor)
}

/// 辅助 trait 用于 pipe 操作
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl Pipe for f64 {}

/// 开仓信号 V2 - 包含 Valuescan 评分系统
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradingSignalV2 {
    pub signal: String,           // "BUY", "SELL", "SKIP"
    pub confidence: String,       // "HIGH", "MEDIUM", "LOW"
    pub entry_price: Option<f64>, // 改为 Option，SKIP 时可为 null
    pub stop_loss: Option<f64>,   // 改为 Option，SKIP 时可为 null
    pub target_price: Option<f64>,
    pub risk_reward_ratio: Option<f64>, // 改为 Option，SKIP 时可为 null
    pub position_size_pct: f64,
    pub reason: String,
    #[serde(default)]
    pub key_levels: Option<KeyLevels>, // ✅ AI可选字段
    pub valuescan_score: f64, // 0-10 分
    #[serde(default)]
    pub score_breakdown: Option<ScoreBreakdown>, // ✅ AI可选字段
    #[serde(default)]
    pub risk_warnings: Vec<String>, // ✅ 默认空数组
    #[serde(default = "default_coin_type")]
    pub coin_type: String, // "mainstream", "altcoin"
    #[serde(default)]
    pub strategy_adjustments: Option<StrategyAdjustments>, // ✅ AI可选字段
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyLevels {
    #[serde(deserialize_with = "deserialize_flexible_f64")]
    pub resistance: f64,
    #[serde(deserialize_with = "deserialize_flexible_f64")]
    pub support: f64,
    #[serde(default)]
    pub current_position: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScoreBreakdown {
    #[serde(rename = "关键位突破", default)]
    pub key_level_breakout: f64,
    #[serde(rename = "资金流向确认", default)]
    pub fund_flow_confirm: f64,
    #[serde(rename = "位置合理", alias = "位置合理与风险收益比", default)]
    pub position_reasonable: f64,
    #[serde(rename = "K线形态配合", default)]
    pub kline_pattern: f64,
    #[serde(rename = "技术指标配合", default)]
    pub technical_indicator: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StrategyAdjustments {
    pub volume_threshold: f64, // 成交量阈值倍数
    pub stop_loss_buffer: f64, // 止损缓冲百分比
    pub max_hold_time: String, // "无限制" 或 "12-24h"
}

/// 持仓管理决策 V2 - 包含 Valuescan 关键位止盈法
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PositionManagementDecisionV2 {
    pub action: String, // "PARTIAL_CLOSE", "FULL_CLOSE", "HOLD"
    pub close_percentage: Option<f64>,
    pub reason: String,
    pub key_analysis: KeyAnalysis,
    pub optimal_exit_price: Option<f64>,
    pub remaining_target: Option<f64>,
    pub new_stop_loss: Option<f64>,
    pub confidence: String,
    pub valuescan_score: f64,
    pub score_breakdown: PositionScoreBreakdown,
    pub risk_warnings: Vec<String>,
    pub hold_conditions_check: HoldConditionsCheck,
    pub decision_priority: DecisionPriority,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyAnalysis {
    pub resistance_distance: String,
    pub support_distance: String,
    pub reversal_signals: Vec<String>,
    pub profit_level: f64,
    pub peak_profit: Option<f64>,
    pub drawdown: Option<f64>,
    pub hold_duration: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PositionScoreBreakdown {
    #[serde(rename = "关键位判断")]
    pub key_level_judgment: f64,
    #[serde(rename = "反转信号确认")]
    pub reversal_signal_confirm: f64,
    #[serde(rename = "盈利保护合理")]
    pub profit_protection: f64,
    #[serde(rename = "风险控制到位")]
    pub risk_control: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HoldConditionsCheck {
    #[serde(rename = "距离阻力>3%")]
    pub distance_to_resistance: bool,
    #[serde(rename = "无反转K线")]
    pub no_reversal_kline: bool,
    #[serde(rename = "多周期共振")]
    pub multi_period_resonance: bool,
    #[serde(rename = "成交量健康")]
    pub healthy_volume: bool,
    #[serde(rename = "时间成本合理")]
    pub reasonable_time_cost: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DecisionPriority {
    pub level: i32, // 1=关键位, 2=K线反转, 3=盈利时间
    pub reason: String,
}

/// 从 V2 转换为 V1 格式(向后兼容)
impl From<TradingSignalV2> for crate::deepseek_client::TradingSignal {
    fn from(v2: TradingSignalV2) -> Self {
        crate::deepseek_client::TradingSignal {
            signal: v2.signal,
            confidence: v2.confidence,
            entry_price: v2.entry_price, // 已经是 Option<f64>
            stop_loss: v2.stop_loss,     // 已经是 Option<f64>
            take_profit: v2.target_price,
            reason: format!(
                "{} (Valuescan评分: {:.1}/10)",
                v2.reason, v2.valuescan_score
            ),
        }
    }
}

/// 从 V2 转换为 V1 持仓管理决策(向后兼容)
impl From<PositionManagementDecisionV2> for crate::deepseek_client::PositionManagementDecision {
    fn from(v2: PositionManagementDecisionV2) -> Self {
        crate::deepseek_client::PositionManagementDecision {
            action: v2.action,
            close_percentage: v2.close_percentage,
            limit_price: None,
            reason: format!(
                "{} (优先级{}: {})",
                v2.reason, v2.decision_priority.level, v2.decision_priority.reason
            ),
            profit_potential: if let Some(target) = v2.remaining_target {
                format!("+{:.1}%", target)
            } else {
                "NONE".to_string()
            },
            optimal_exit_price: v2.optimal_exit_price,
            confidence: v2.confidence,
            stop_loss_adjustment: v2.new_stop_loss.map(|price| {
                crate::deepseek_client::StopLossAdjustment {
                    should_adjust: true,
                    new_stop_loss: Some(price),
                    reason: "基于关键位调整".to_string(),
                }
            }),
            take_profit_adjustment: None,
        }
    }
}
