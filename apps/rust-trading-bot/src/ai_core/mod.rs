//! AI 统一接口与协同决策模块。
//!
//! `ai_trait` 定义所有 AI 提供者必须实现的统一数据结构与 trait，
//! AI Core Module
//!
//! 统一的AI client接口和多个提供商实现

pub mod ai_trait;
pub mod decision_engine;
pub mod deepseek;
pub mod gemini;
pub mod prompt_builder;
pub mod prompt_contexts;
pub mod traits;

pub use ai_trait::{
    AIProvider, EntryAction, EntryContext, EntryDecision, EntrySignal, PositionAction,
    PositionContext, PositionDecision, StopLossAdjustmentDecision, TakeProfitAdjustmentDecision,
};
pub use decision_engine::DecisionEngine;
pub use decision_engine::DecisionEngine as AIDecisionEngine;
pub use decision_engine::{DecisionEngineConfig, EntryConsensus, PositionConsensus};
pub use prompt_builder::*;
pub use prompt_contexts::*;
