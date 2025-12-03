//! 核心调度模块
//!
//! - `entry_manager` 负责入场信号接入与交易执行入口
//! - `position_manager` 作为持仓监控门面, 逐步承接 `IntegratedAITrader` 的监控职责

pub mod entry_manager;
pub mod position_manager;
pub mod risk_controller;
pub mod signal_processor;

pub use entry_manager::EntryManager;
pub use position_manager::PositionManagerFacade;
pub use risk_controller::RiskController;
pub use signal_processor::SignalProcessor;
