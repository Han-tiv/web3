//! 集成AI交易系统模块
//!
//! 将原 trader.rs 拆分为多个职责清晰的模块

pub mod config;
pub mod types;

// 重新导出所有类型，便于 trader.rs 和其他模块使用
pub use config::*;
pub use types::*;
