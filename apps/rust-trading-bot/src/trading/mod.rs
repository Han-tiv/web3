//! 交易相关的通用组件。

pub mod order_manager;
pub mod position_manager;

pub use order_manager::{OrderManager, OrderManagerConfig};
pub use position_manager::{PositionAction, PositionManager, PositionTracker, TrackerMutation};
