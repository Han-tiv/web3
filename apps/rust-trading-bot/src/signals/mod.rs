//! 信号解析与分类模块。

pub mod alert_classifier;
pub mod message_parser;

pub use alert_classifier::{AlertType, FundAlert};
pub use message_parser::{MessageParser, SignalContext};
