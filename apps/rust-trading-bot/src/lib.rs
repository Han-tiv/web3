pub mod binance_client;
pub mod copy_trader;
// pub mod signals;  // 已删除：功能已集成
pub mod telegram_bot;
pub mod telegram_notifier;
pub mod telegram_signal; // Telegram信号评分系统
// pub mod trading;  // 已删除：功能已集成
pub mod trading_lock;

// 交易所客户端模块
// pub mod bitget_client;   // 已删除
// pub mod bybit_client;    // 已删除
pub mod exchange_trait;
// pub mod gate_client;     // 已删除
pub mod hyperliquid_client;
// pub mod okx_client;      // 已删除

// 区块链钱包模块
// pub mod bsc_wallet;      // 已删除
// pub mod solana_wallet;   // 已删除

// 价格服务
pub mod price_service;

// 多交易所执行器
// pub mod multi_exchange_executor;  // 已删除

// DeepSeek AI 交易模块（纯技术指标版本）
// pub mod ai;  // 已删除：功能已集成
pub mod deepseek_client;
pub mod gemini_client;
// pub mod grok_client;  // 已删除
pub mod technical_analysis;
pub mod valuescan_v2; // Valuescan V2 数据结构
                      // pub mod market_sentiment;        // 已移除：不使用情绪分析
                      // pub mod crypto_oracle_client;    // 已移除：不使用外部情绪数据

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
