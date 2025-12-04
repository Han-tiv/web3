//! Exchanges Module
//!
//! 统一的交易所接口和多个交易所的实现
//!
//! # 模块结构
//! - `traits` - Exchange trait 定义
//! - `binance` - Binance交易所实现
//! - `bybit` - Bybit交易所实现
//! - `okx` - OKX交易所实现
//! - `gate` - Gate.io交易所实现
//! - `bitget` - Bitget交易所实现
//! - `hyperliquid` - Hyperliquid交易所实现
//! - `multi_executor` - 多交易所执行器

pub mod traits;

pub mod binance;
pub mod bybit;
pub mod okx;
pub mod gate;
pub mod bitget;
pub mod hyperliquid;

pub mod multi_executor;

// 重新导出主要类型
pub use traits::{
    AccountInfo, ExchangeClient, ExchangeType, OrderResult, Position, TradingRules,
};
pub use multi_executor::MultiExchangeExecutor;
