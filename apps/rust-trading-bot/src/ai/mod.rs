//! AI 统一接口与协同决策模块。
//!
//! `ai_trait` 定义所有 AI 提供者必须实现的统一数据结构与 trait，
//! `decision_engine` 则负责聚合多个 AI 提供者的结果，并输出最终共识。

pub mod ai_trait;
pub mod decision_engine;

pub use ai_trait::{
    AIProvider, EntryAction, EntryContext, EntryDecision, EntrySignal, PositionAction,
    PositionContext, PositionDecision, StopLossAdjustmentDecision, TakeProfitAdjustmentDecision,
};
pub use decision_engine::{
    DecisionEngine, DecisionEngineConfig, EntryConsensus, PositionConsensus,
};
