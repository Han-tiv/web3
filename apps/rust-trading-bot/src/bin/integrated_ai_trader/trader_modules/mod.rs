//! Trader Modules
//!
//! 将trader.rs的功能拆分为多个职责清晰的模块

pub mod state_manager;
pub mod signal_handler;

pub use state_manager::{StateManager, VolatilityCacheEntry};
pub use signal_handler::SignalHandler;
