//! 集成AI交易系统数据类型定义
//!
//! 从 trader.rs 提取的所有数据结构

use chrono::{DateTime, Utc};
use rust_trading_bot::binance_client::BinanceClient;
use rust_trading_bot::database::Database;
pub use rust_trading_bot::deepseek_client::TechnicalIndicators;
use rust_trading_bot::deepseek_client::{DeepSeekClient, Kline};
use rust_trading_bot::entry_zone_analyzer::{EntryDecision, EntryZone, EntryZoneAnalyzer};
use rust_trading_bot::gemini_client::GeminiClient;
pub use rust_trading_bot::prompt_contexts::{EntryPromptContext, PositionPromptContext};
use rust_trading_bot::signals::FundAlert;
use rust_trading_bot::staged_position_manager::StagedPositionManager;
use rust_trading_bot::technical_analysis::TechnicalAnalyzer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Entry 管理器初始化所需的完整配置
pub struct EntryManagerConfig {
    pub exchange: Arc<BinanceClient>,
    pub deepseek: Arc<DeepSeekClient>,
    pub gemini: Arc<GeminiClient>,
    pub analyzer: Arc<TechnicalAnalyzer>,
    pub entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>,
    pub signal_history: Arc<RwLock<SignalHistory>>,
    pub last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    pub risk_limits: RiskLimitConfig,
    pub db: Database,
}

/// 风险阈值配置
pub struct RiskLimitConfig {
    pub max_position_usdt: f64,
    pub min_position_usdt: f64,
    pub max_leverage: u32,
    pub min_leverage: u32,
}

/// 执行AI试探建仓所需的上下文
pub struct EntryExecutionRequest<'a> {
    pub symbol: &'a str,
    pub alert: &'a FundAlert,
    pub zone_1h: &'a EntryZone,
    pub entry_decision: &'a EntryDecision,
    pub klines_15m: &'a [Kline],
    pub klines_5m: &'a [Kline],
    pub current_price: f64,
    pub final_entry_price: f64,
    pub final_stop_loss: f64,
    pub final_confidence: &'a str,
    pub ai_position_multiplier: f64,
    pub ai_signal_side: &'a str,
    pub take_profit: Option<f64>,
    pub is_ai_override: bool,
}

/// 延迟开仓队列记录 - 首次未开仓的币种,等待更好时机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEntry {
    pub symbol: String,
    pub first_signal_time: DateTime<Utc>,
    pub last_analysis_time: DateTime<Utc>,
    pub alert: FundAlert,
    pub reject_reason: String, // 为什么首次被拒绝: "价格不符"/"AI SKIP"/"等待回调"
    pub retry_count: u32,      // 已重试次数
    pub fund_escape_detected_at: Option<DateTime<Utc>>, // 首次检测到资金出逃的时间
}

/// 持仓追踪器快照 - 用于无锁读取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSnapshot {
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

/// 持倉追蹤資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 缓存批量AI评估所需的行情上下文，避免重复获取K线
pub struct PositionMarketContext {
    pub klines_5m: Vec<Kline>,
    pub klines_15m: Vec<Kline>,
    pub klines_1h: Vec<Kline>,
    pub indicators: TechnicalIndicators,
}

/// 保存批量AI评估完成后执行交易动作所需的持仓信息
pub struct BatchActionContext {
    pub side: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
}

/// 批量AI评估输入结构，封装单个持仓的关键上下文
#[derive(Debug, Clone)]
pub struct BatchPositionInput {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub klines_5m: Vec<Kline>,
    pub klines_15m: Vec<Kline>,
    pub klines_1h: Vec<Kline>,
    pub indicators: TechnicalIndicators,
}

impl From<BatchPositionInput>
    for (
        String,
        String,
        f64,
        f64,
        f64,
        f64,
        Vec<Kline>,
        Vec<Kline>,
        Vec<Kline>,
        TechnicalIndicators,
    )
{
    fn from(value: BatchPositionInput) -> Self {
        (
            value.symbol,
            value.side,
            value.entry_price,
            value.stop_loss_price,
            value.current_price,
            value.quantity,
            value.klines_5m,
            value.klines_15m,
            value.klines_1h,
            value.indicators,
        )
    }
}

/// 统一封装AI评估所需的完整上下文，复用单次与批量流程
pub struct PreparedPositionContext {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub duration: f64,
    pub profit_pct: f64,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub min_notional: f64,
    pub market: PositionMarketContext,
    pub support_text: String,
    pub deviation_desc: String,
    pub current_stop_loss: Option<f64>,
    pub current_take_profit: Option<f64>,
}

/// 波动率缓存条目
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct VolatilityCacheEntry {
    pub value: f64,
    pub cached_at: Instant,
}

/// 触发单跟踪记录
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TriggerOrderRecord {
    pub order_id: String,
    pub symbol: String,
    pub position_side: String,
    pub trigger_price: f64,
    pub action: String, // "OPEN" or "CLOSE"
    pub created_at: DateTime<Utc>,
    pub reason: String,
}

/// 持仓监控阶段需要执行的动作，采用"先收集再处理"策略避免锁重入
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
        stop_loss_price: f64, // ✅ Bug Fix: 保存原始止损价格,部分平仓后重设止损单使用
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

/// 描述AI分析前的准备结果
#[allow(clippy::large_enum_variant)]
pub enum PositionEvaluationStep {
    Skip,
    Immediate(PositionAction),
    Context(PreparedPositionContext),
}

/// 对追踪器的更新操作，统一在短暂写锁中落盘
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

/// 交易信號記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRecord {
    pub timestamp: String,
    pub signal: String,
    pub confidence: String,
    pub reason: String,
    pub price: f64,
}

/// 交易信號歷史
pub struct SignalHistory {
    pub signals: VecDeque<SignalRecord>,
    pub max_size: usize,
}

impl SignalHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            signals: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add(&mut self, record: SignalRecord) {
        if self.signals.len() >= self.max_size {
            self.signals.pop_front();
        }
        self.signals.push_back(record);
    }

    #[allow(dead_code)]
    pub fn get_recent(&self, count: usize) -> Vec<&SignalRecord> {
        self.signals.iter().rev().take(count).collect()
    }

    #[allow(dead_code)]
    pub fn count_signal(&self, signal: &str, last_n: usize) -> usize {
        self.signals
            .iter()
            .rev()
            .take(last_n)
            .filter(|s| s.signal == signal)
            .count()
    }
}

impl PreparedPositionContext {
    /// 将持仓上下文压缩为 prompt 字符串，方便传递给不同 AI 客户端
    pub fn to_prompt_context(&self) -> String {
        format!(
            concat!(
                "Symbol: {}\n",
                "Side: {}\n",
                "Entry Price: {:.4}\n",
                "Current Price: {:.4}\n",
                "Stop Loss: {:.4}\n",
                "Quantity: {:.4}\n",
                "Duration Hours: {:.2}\n",
                "PnL %: {:.2}\n",
                "Support: {}\n",
                "Deviation: {}\n",
                "Current SL: {}\n",
                "Current TP: {}\n"
            ),
            self.symbol,
            self.side,
            self.entry_price,
            self.current_price,
            self.stop_loss_price,
            self.quantity,
            self.duration,
            self.profit_pct,
            self.support_text,
            self.deviation_desc,
            self.current_stop_loss
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "未设置".to_string()),
            self.current_take_profit
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "未设置".to_string())
        )
    }

    /// 转换为批量AI评估输入，包含K线和技术指标快照
    pub fn to_batch_input(&self) -> BatchPositionInput {
        BatchPositionInput {
            symbol: self.symbol.clone(),
            side: self.side.clone(),
            entry_price: self.entry_price,
            stop_loss_price: self.stop_loss_price,
            current_price: self.current_price,
            quantity: self.quantity,
            klines_5m: self.market.klines_5m.clone(),
            klines_15m: self.market.klines_15m.clone(),
            klines_1h: self.market.klines_1h.clone(),
            indicators: self.market.indicators.clone(),
        }
    }
}

/// Phase 2.3: 持仓上下文请求 - 传递给ContextBuilder和PositionEvaluator
/// 降低prepare_position_context和evaluate方法的参数数量(9→1)
pub struct PositionContextRequest<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub duration_hours: f64,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
}
