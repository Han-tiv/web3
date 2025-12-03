//! 数据模块
//!
//! 集中管理持仓追踪器、波动率缓存与交易历史记录，提供线程安全的封装。

pub mod cache_manager;
pub mod history_recorder;
pub mod tracker_manager;

pub use cache_manager::CacheManager;
pub use history_recorder::{HistoryRecorder, TradeRecordParams};
pub use tracker_manager::TrackerManager;
