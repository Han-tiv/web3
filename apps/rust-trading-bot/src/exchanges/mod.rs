//! Exchanges Module
//!
//! 统一的交易所接口和实现
//!
//! # 模块结构
//! - `traits` - Exchange trait 定义
//! - `binance` - Binance交易所实现

pub mod traits;
pub mod binance;

// 重新导出常用类型
pub use binance::BinanceClient;
pub use traits::{
    AccountInfo, ExchangeClient, ExchangeType, OrderResult, Position, TradingRules,
};
