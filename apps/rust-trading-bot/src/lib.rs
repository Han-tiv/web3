//! Rust Trading Bot Library
//!
//! 模块化的加密货币交易机器人核心库
//!
//! # 模块结构
//!
//! - `exchanges` - 交易所客户端 (Binance)
//! - `api` - Web API服务器
//! - `config` - 配置和数据库
//! - `ai_core` - AI客户端 (DeepSeek, Gemini)
//! - `trading_core` - 交易核心逻辑 (信号、执行、仓位管理)
//! - `analysis` - 技术分析和市场数据
//! - `integrations` - 第三方集成 (Telegram, Valuescan)
//! - `utils` - 通用工具

// 核心模块
pub mod exchanges;
pub mod api;
pub mod config;
pub mod errors;        // ✨ 统一错误处理
pub mod services;      // ✨ 服务层
pub mod domain;        // ✨ 领域模型
pub mod state;         // ✨ 状态管理
pub mod repositories;  // ✨ 数据访问层
pub mod ai_core;
pub mod trading_core;
pub mod analysis;

// 集成和工具
pub mod integrations;
pub mod utils;

// 保留原有的模块导出以兼容现有代码
pub use exchanges::binance::BinanceClient;
pub use config::Database;
pub use integrations::telegram as telegram_bot;
pub use integrations::telegram as telegram_notifier;
pub use integrations::telegram as telegram_signal;
pub use trading_core::signals;
pub mod trading {
    //! Trading module - keeping for backward compatibility
    pub use crate::trading_core::*;
}
pub use trading_core::execution::lock as trading_lock;

// 重新导出常用类型
pub use exchanges::{ExchangeClient, Position, AccountInfo, OrderResult};
pub use config::Database as database;
pub use integrations::price_service;
pub use api::server as web_server;

// Keep old module names for backward compatibility temporarily
pub use exchanges::binance::BinanceClient as binance_client;
pub use exchanges::bybit::BybitClient;
pub use exchanges::okx::OkxClient;
pub use exchanges::gate::GateClient;
pub use exchanges::bitget::BitgetClient;
pub use exchanges::hyperliquid::HyperliquidClient;

pub use ai_core::deepseek;
pub use ai_core::gemini;
// AI clients
pub use ai_core::deepseek as deepseek_client;
pub use ai_core::gemini as gemini_client;
// Removed: grok_client (deleted)
pub use ai_core::decision_engine as ai_decision_engine;
pub use ai_core::prompt_contexts;

pub use analysis::technical as technical_analysis;
pub use analysis::technical::TechnicalAnalyzer as TechnicalAnalysis;
pub use analysis::market_data as market_data_fetcher;
pub use analysis::key_levels as key_level_finder;
pub use analysis::support as support_analyzer;
pub use analysis::entry_zone as entry_zone_analyzer;
pub use analysis::smart_money as smart_money_tracker;
pub use analysis::launch_signals as launch_signal_detector;

pub use trading_core::execution::executor as trade_executor;
pub use trading_core::signals::manager as signal_manager;
pub use trading_core::positions::coordinator as position_coordinator;
pub use trading_core::positions::staged_manager as staged_position_manager;
pub use trading_core::copy_trader;

pub use integrations::valuescan as valuescan_v2;
// Removed: wallets module (deleted)
pub use utils::health_monitor;
pub use utils::coin_parser;

// Keep multi_exchange_executor visible
// Removed: multi_exchange_executor (requires deleted exchanges)

// Keep exchange_trait visible  
pub use exchanges::traits as exchange_trait;

// Keep ai submodule visible
pub use trading_core::signals as ai;
