//! Trading Core Module
//!
//! 信号处理、订单执行、仓位管理

pub mod signals;
pub mod execution;
pub mod positions;
pub mod copy_trader;

pub use signals::SignalManager;
pub use execution::TradeExecutor;
pub use positions::PositionCoordinator;
pub use copy_trader::CopyTrader;
