//! AI 评估模块
//!
//! - `context_builder`: 负责构建持仓评估上下文及快速规则判断
//! - `decision_handler`: 负责解析 AI 决策并转换为交易动作
//! - `evaluator`: 串联上下文构建、AI 调用与决策落地
//! - `kline_fetcher`: 多周期K线获取
//! - `entry_analyzer`: 入场区分析
//! - `ai_decider`: AI 入场决策模块

pub mod ai_decider;
pub mod context_builder;
pub mod decision_handler;
pub mod entry_analyzer;
pub mod evaluator;
pub mod kline_fetcher;

pub use ai_decider::AIDecider;
pub use context_builder::ContextBuilder;
pub use decision_handler::{ActionBuildParams, DecisionHandler};
pub use entry_analyzer::EntryAnalyzer;
pub use evaluator::PositionEvaluator;
pub use kline_fetcher::KlineFetcher;
