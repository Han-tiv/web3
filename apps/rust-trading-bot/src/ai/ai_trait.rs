use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 入场分析上下文，将主流程中准备好的 prompt、币种信息等封装为统一结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryContext {
    pub symbol: String,
    pub timeframe: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub metadata: Value,
}

impl EntryContext {
    pub fn new<S: Into<String>, P: Into<String>>(symbol: S, prompt: P) -> Self {
        Self {
            symbol: symbol.into(),
            timeframe: None,
            prompt: prompt.into(),
            metadata: Value::Null,
        }
    }

    pub fn with_timeframe<S: Into<String>>(mut self, timeframe: S) -> Self {
        self.timeframe = Some(timeframe.into());
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 持仓分析上下文，提供仓位尺寸、方向等关键信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionContext {
    pub symbol: String,
    pub side: Option<String>,
    pub entry_price: Option<f64>,
    pub current_price: Option<f64>,
    pub quantity: Option<f64>,
    pub duration_hours: Option<f64>,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub metadata: Value,
}

impl PositionContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new<S: Into<String>, P: Into<String>>(
        symbol: S,
        side: Option<String>,
        entry_price: Option<f64>,
        current_price: Option<f64>,
        quantity: Option<f64>,
        duration_hours: Option<f64>,
        prompt: P,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            entry_price,
            current_price,
            quantity,
            duration_hours,
            stop_loss_order_id: None,
            take_profit_order_id: None,
            prompt: prompt.into(),
            metadata: Value::Null,
        }
    }

    pub fn with_protection_orders(
        mut self,
        stop_loss_order_id: Option<String>,
        take_profit_order_id: Option<String>,
    ) -> Self {
        self.stop_loss_order_id = stop_loss_order_id;
        self.take_profit_order_id = take_profit_order_id;
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// AI 交易信号的通用结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySignal {
    pub label: String,
    pub reason: String,
    pub confidence: Option<String>,
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

impl EntrySignal {
    pub fn confidence_score(&self) -> u8 {
        score_from_confidence(self.confidence.as_deref())
    }
}

/// AI 入场决策，包含标准化动作与原始信号信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryDecision {
    pub provider: String,
    pub symbol: String,
    pub action: EntryAction,
    pub signal: EntrySignal,
    pub metadata: Value,
    pub raw_response: Option<String>,
}

impl EntryDecision {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: &str,
        symbol: &str,
        signal_label: impl Into<String>,
        reason: impl Into<String>,
        confidence: Option<String>,
        entry_price: Option<f64>,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
        metadata: Option<Value>,
        raw_response: Option<String>,
    ) -> Self {
        let label = signal_label.into();
        let signal = EntrySignal {
            label: label.clone(),
            reason: reason.into(),
            confidence,
            entry_price,
            stop_loss,
            take_profit,
        };

        Self {
            provider: provider.to_string(),
            symbol: symbol.to_string(),
            action: EntryAction::from_label(&label),
            signal,
            metadata: metadata.unwrap_or(Value::Null),
            raw_response,
        }
    }

    pub fn confidence_score(&self) -> u8 {
        self.signal.confidence_score()
    }
}

/// 统一的入场动作。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryAction {
    Buy,
    Sell,
    Hold,
    Skip,
    Custom(String),
}

impl EntryAction {
    pub fn from_label<S: AsRef<str>>(label: S) -> Self {
        let trimmed = label.as_ref().trim();
        let normalized = trimmed.to_ascii_uppercase();
        match normalized.as_str() {
            "BUY" | "LONG" => EntryAction::Buy,
            "SELL" | "SHORT" => EntryAction::Sell,
            "HOLD" => EntryAction::Hold,
            "SKIP" => EntryAction::Skip,
            _ => EntryAction::Custom(trimmed.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EntryAction::Buy => "BUY",
            EntryAction::Sell => "SELL",
            EntryAction::Hold => "HOLD",
            EntryAction::Skip => "SKIP",
            EntryAction::Custom(value) => value.as_str(),
        }
    }

    pub fn key(&self) -> String {
        self.as_str().to_ascii_uppercase()
    }
}

/// 止损调整信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossAdjustmentDecision {
    pub should_adjust: bool,
    pub new_stop_loss: Option<f64>,
    pub reason: String,
}

impl StopLossAdjustmentDecision {
    pub fn new<R: Into<String>>(
        should_adjust: bool,
        new_stop_loss: Option<f64>,
        reason: R,
    ) -> Self {
        Self {
            should_adjust,
            new_stop_loss,
            reason: reason.into(),
        }
    }
}

/// 止盈调整信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitAdjustmentDecision {
    pub should_adjust: bool,
    pub new_take_profit: Option<f64>,
    pub reason: String,
}

impl TakeProfitAdjustmentDecision {
    pub fn new<R: Into<String>>(
        should_adjust: bool,
        new_take_profit: Option<f64>,
        reason: R,
    ) -> Self {
        Self {
            should_adjust,
            new_take_profit,
            reason: reason.into(),
        }
    }
}

/// AI 的持仓决策。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDecision {
    pub provider: String,
    pub symbol: String,
    pub action: PositionAction,
    pub reason: String,
    pub confidence: Option<String>,
    pub profit_potential: Option<String>,
    pub close_percentage: Option<f64>,
    pub limit_price: Option<f64>,
    pub optimal_exit_price: Option<f64>,
    pub stop_loss_adjustment: Option<StopLossAdjustmentDecision>,
    pub take_profit_adjustment: Option<TakeProfitAdjustmentDecision>,
    pub metadata: Value,
    pub raw_response: Option<String>,
}

impl PositionDecision {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: &str,
        symbol: &str,
        action_label: impl Into<String>,
        reason: impl Into<String>,
        confidence: Option<String>,
        profit_potential: Option<String>,
        close_percentage: Option<f64>,
        limit_price: Option<f64>,
        optimal_exit_price: Option<f64>,
        stop_loss_adjustment: Option<StopLossAdjustmentDecision>,
        take_profit_adjustment: Option<TakeProfitAdjustmentDecision>,
        metadata: Option<Value>,
        raw_response: Option<String>,
    ) -> Self {
        let label = action_label.into();
        Self {
            provider: provider.to_string(),
            symbol: symbol.to_string(),
            action: PositionAction::from_label(&label),
            reason: reason.into(),
            confidence,
            profit_potential,
            close_percentage,
            limit_price,
            optimal_exit_price,
            stop_loss_adjustment,
            take_profit_adjustment,
            metadata: metadata.unwrap_or(Value::Null),
            raw_response,
        }
    }

    pub fn confidence_score(&self) -> u8 {
        score_from_confidence(self.confidence.as_deref())
    }
}

/// 统一的持仓动作。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionAction {
    Hold,
    PartialClose,
    FullClose,
    SetLimitOrder,
    AdjustProtection,
    Custom(String),
}

impl PositionAction {
    pub fn from_label<S: AsRef<str>>(label: S) -> Self {
        let trimmed = label.as_ref().trim();
        let normalized = trimmed.to_ascii_uppercase();
        match normalized.as_str() {
            "HOLD" => PositionAction::Hold,
            "PARTIAL_CLOSE" | "PARTIAL_EXIT" => PositionAction::PartialClose,
            "FULL_CLOSE" | "EXIT_ALL" => PositionAction::FullClose,
            "SET_LIMIT_ORDER" | "LIMIT_ORDER" => PositionAction::SetLimitOrder,
            "ADJUST_PROTECTION" => PositionAction::AdjustProtection,
            _ => PositionAction::Custom(trimmed.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PositionAction::Hold => "HOLD",
            PositionAction::PartialClose => "PARTIAL_CLOSE",
            PositionAction::FullClose => "FULL_CLOSE",
            PositionAction::SetLimitOrder => "SET_LIMIT_ORDER",
            PositionAction::AdjustProtection => "ADJUST_PROTECTION",
            PositionAction::Custom(value) => value.as_str(),
        }
    }

    pub fn key(&self) -> String {
        self.as_str().to_ascii_uppercase()
    }
}

fn score_from_confidence(confidence: Option<&str>) -> u8 {
    match confidence.map(|s| s.trim().to_ascii_uppercase()) {
        Some(ref value) if value == "HIGH" => 3,
        Some(ref value) if value == "MEDIUM" => 2,
        Some(ref value) if value == "LOW" => 1,
        _ => 0,
    }
}

/// 所有 AI 客户端必须实现的统一 trait。
#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn analyze_entry(&self, ctx: &EntryContext) -> Result<EntryDecision>;

    async fn analyze_position(&self, ctx: &PositionContext) -> Result<PositionDecision>;
}
