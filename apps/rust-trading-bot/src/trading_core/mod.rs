//! Trading Core Module
//!
//! 信号处理、订单执行、仓位管理

pub mod copy_trader;
pub mod execution;
pub mod positions;
pub mod signals;

pub use copy_trader::CopyTrader;
pub use execution::TradeExecutor;
pub use positions::PositionCoordinator;
pub use signals::SignalManager;
