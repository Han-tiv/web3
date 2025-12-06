// 核心交易模块
pub mod binance_client;
pub mod copy_trader;
pub mod telegram_bot;
pub mod telegram_notifier;
pub mod telegram_signal;
pub mod trading_lock;

// 交易所客户端
pub mod exchange_trait;
pub mod hyperliquid_client;

// 价格服务
pub mod price_service;

// AI 交易模块
pub mod deepseek_client;
pub mod gemini_client;
pub mod technical_analysis;
pub mod valuescan_v2;

// 主力资金追踪交易模块
pub mod key_level_finder; // 关键位识别
pub mod smart_money_tracker; // 主力资金追踪
pub mod support_analyzer; // 完整版支撑位识别系统

// 分批建仓 + 启动补仓策略模块
pub mod entry_zone_analyzer; // 1h+15m入场区分析
pub mod launch_signal_detector; // 启动信号检测
pub mod staged_position_manager; // 分批建仓管理

// 市场数据获取
pub mod market_data_fetcher;

// 健康监控
pub mod health_monitor;

// Web API服务器
pub mod web_server;

// 动态仓位管理系统模块
pub mod ai_decision_engine; // AI批量决策引擎
pub mod coin_parser; // 币种解析器
pub mod database; // SQLite 数据访问层
pub mod position_coordinator; // 仓位协调器
pub mod signal_manager; // 信号队列管理
pub mod trade_executor; // 交易执行器
