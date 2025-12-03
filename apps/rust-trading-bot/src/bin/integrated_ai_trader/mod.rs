//! Integrated AI Trader - 集成AI交易系统
//!
//! 整合主力资金监控 + AI分析 + 多交易所执行的完整交易系统
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Main Coordinator                       │
//! │              (mod.rs - 主入口协调器)                      │
//! └─────────────────────────────────────────────────────────┘
//!                            │
//!        ┌───────────────────┼───────────────────┐
//!        │                   │                   │
//!   ┌────▼────┐         ┌───▼────┐         ┌───▼────┐
//!   │ Trader  │         │ Entry  │         │Position│
//!   │ (State) │         │Analyzer│         │Monitor │
//!   └─────────┘         └────────┘         └────────┘
//!        │                   │                   │
//!   ┌────▼────┐         ┌───▼────┐         ┌───▼────┐
//!   │  Utils  │         │ Entry  │         │Position│
//!   │         │         │Executor│         │Evaluator│
//!   └─────────┘         └────────┘         └────────┘
//!                            │                   │
//!                       ┌────▼────┐         ┌───▼────┐
//!                       │Position │         │ Order  │
//!                       │Operator │         │Monitor │
//!                       └─────────┘         └────────┘
//!                            │
//!                       ┌────▼────┐
//!                       │ Cleanup │
//!                       │ Manager │
//!                       └─────────┘
//! ```
//!
//! ## 模块说明
//!
//! - `trader` - 核心状态管理，定义IntegratedAITrader结构体
//! - `utils` - 工具函数和常量定义
//! - `entry_analyzer` - 入场分析模块（analyze_and_trade）
//! - `entry_executor` - 入场执行模块（execute_ai_trial_entry）
//! - `position_monitor` - 持仓监控主循环（monitor_positions）
//! - `position_evaluator` - 持仓AI评估（evaluate_position_with_ai）
//! - `position_operator` - 持仓操作执行（close_position_fully/partially）
//! - `order_monitor` - 订单监控管理
//! - `cleanup_manager` - 内存和清理管理
//!
//! ## 功能特性
//!
//! 1. **信号接收**: 从Telegram获取Alpha/FOMO信号
//! 2. **AI分析**: Gemini入场分析 + DeepSeek持仓管理
//! 3. **风控管理**: 多层次止损、分批建仓、MEME币特殊风控
//! 4. **持仓监控**: 180秒循环，4阶段管理
//! 5. **订单管理**: 止盈止损互斥、触发单监控
//! 6. **内存管理**: 自动清理过期数据

pub mod ai;
pub mod cleanup_manager;
pub mod core;
pub mod data;
pub mod entry_analyzer;
pub mod entry_executor;
pub mod execution;
pub mod modules;
pub mod order_monitor;
pub mod position_evaluator;
pub mod position_monitor;
pub mod position_operator;
pub mod trader;
pub mod utils;

pub use ai::{ContextBuilder, DecisionHandler, PositionEvaluator};
// 重新导出 trader/modules 下的类型，便于其他模块使用
pub use modules::config::*;
pub use modules::types::*;
pub use trader::IntegratedAITrader;

use anyhow::Result;
use log::{error, info, warn};
use std::env;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use rust_trading_bot::{
    binance_client::BinanceClient,
    database::Database,
    signals::{AlertType, FundAlert, SignalContext},
    web_server,
};

/// 主程序入口
///
/// # 启动流程
/// 1. 加载环境变量
/// 2. 初始化日志系统
/// 3. 创建交易器实例
/// 4. 启动并发任务:
///    - 持仓监控线程 (180秒循环)
///    - 延迟开仓重新分析线程 (600秒循环)
///    - Web服务器 (8080端口)
///    - Telegram信号轮询 (5秒循环)
#[tokio::main]
pub async fn main() -> Result<()> {
    // 从web3根目录加载环境变量
    dotenv::from_path("/home/hanins/code/web3/.env").ok();

    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 打印启动信息
    print_startup_banner();

    // 读取配置
    let config = load_configuration()?;

    // 初始化Binance客户端
    let exchange = BinanceClient::new(
        config.binance_api_key,
        config.binance_secret,
        config.testnet,
    );
    info!("✅ Binance客户端已初始化\n");

    // 初始化数据库
    let db = initialize_database()?;
    info!("✅ 数据库已初始化\n");

    // 创建集成交易器
    let trader = IntegratedAITrader::new(
        exchange.clone(),
        config.deepseek_api_key,
        config.gemini_api_key,
        db.clone(),
    )
    .await?;

    // 恢复启动前已存在的持仓
    if let Err(e) = trader.sync_existing_positions().await {
        warn!("⚠️  恢复历史持仓失败: {}", e);
    }

    // 启动并发任务
    spawn_concurrent_tasks(trader, db, config.initial_balance).await?;

    Ok(())
}

/// 打印启动横幅
fn print_startup_banner() {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 集成AI交易系统启动");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📦 版本: 2.0.0-refactored");
    info!("🏗️  架构: 模块化 (10个独立模块)");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// 配置结构
struct Configuration {
    deepseek_api_key: String,
    gemini_api_key: String,
    binance_api_key: String,
    binance_secret: String,
    testnet: bool,
    initial_balance: f64,
}

/// 加载配置
fn load_configuration() -> Result<Configuration> {
    let deepseek_api_key = env::var("DEEPSEEK_API_KEY")?;
    let gemini_api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        warn!("⚠️  GEMINI_API_KEY 未设置，Gemini 入场分析将被禁用");
        String::new()
    });
    let binance_api_key = env::var("BINANCE_API_KEY")?;
    let binance_secret = env::var("BINANCE_SECRET")?;
    let testnet = env::var("BINANCE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    info!("🎯 系统配置:");
    info!("  信号来源: Python Telegram Monitor → Web API /api/signals");
    info!("  监控类型: Alpha机会 + FOMO信号");
    info!("  交易策略: 主力关键位 + 日内波段");
    info!("  AI引擎: DeepSeek(入场分析V2) + Gemini(持仓管理-批量评估)");
    info!("  交易所: Binance");
    info!("  测试模式: {}\n", if testnet { "是" } else { "否" });

    Ok(Configuration {
        deepseek_api_key,
        gemini_api_key,
        binance_api_key,
        binance_secret,
        testnet,
        initial_balance: 50.03,
    })
}

/// 初始化数据库
fn initialize_database() -> Result<Database> {
    let db_path = "data/trading.db";
    info!("📁 初始化数据库: {}", db_path);
    std::fs::create_dir_all("data").ok();
    Database::new(db_path).map_err(|e| anyhow::anyhow!("数据库初始化失败: {}", e))
}

/// 启动并发任务
async fn spawn_concurrent_tasks(
    trader: Arc<IntegratedAITrader>,
    db: Database,
    initial_balance: f64,
) -> Result<()> {
    // 任务1: 持仓监控线程
    let monitor_trader = trader.clone();
    tokio::spawn(async move {
        info!("🔍 持仓监控线程启动");
        monitor_trader.monitor_positions().await;
    });
    info!("✅ 持仓监控线程已启动\n");

    // 任务2: 延迟开仓队列重新分析线程
    let reanalyze_trader = trader.clone();
    tokio::spawn(async move {
        info!("🔄 延迟开仓队列重新分析线程启动");
        reanalyze_trader.reanalyze_pending_entries().await;
    });
    info!("✅ 延迟开仓队列重新分析线程已启动（每3.5分钟）\n");

    // 任务3: Web服务器
    info!("✅ 初始合约余额（固定）: {} USDT", initial_balance);
    let web_server_state = Arc::new(web_server::AppState::new(
        initial_balance,
        db.clone(),
        trader.exchange.clone(),
    ));
    tokio::spawn(async move {
        if let Err(err) = web_server::start_web_server(8080, web_server_state).await {
            error!("❌ Web 服务器启动失败: {:?}", err);
        }
    });
    info!("✅ Web 服务器已启动 (端口 8080)\n");

    // 任务4: Telegram信号轮询
    let trader_for_signals = trader.clone();
    let polling_db = db;
    tokio::spawn(async move {
        let poll_interval = StdDuration::from_secs(5);
        info!("📡 Telegram信号轮询线程启动");

        loop {
            tokio::time::sleep(poll_interval).await;

            match polling_db.list_unprocessed_telegram_signals(100) {
                Ok(records) => {
                    if !records.is_empty() {
                        info!("📡 轮询到 {} 条待处理的Telegram信号", records.len());
                    }

                    for record in records {
                        let Some(record_id) = record.id else {
                            warn!("⚠️ 忽略缺少ID的Telegram信号: {:?}", record.symbol);
                            continue;
                        };

                        // 解析timestamp（从String转为DateTime<Utc>）
                        use chrono::DateTime;
                        let timestamp = DateTime::parse_from_rfc3339(&record.timestamp)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now());

                        // 创建FundAlert，简化为基本信息
                        let alert = FundAlert {
                            coin: record.symbol.clone(),
                            alert_type: AlertType::FundInflow, // 统一类型，由AI决策
                            price: 0.0,                        // 价格将在analyze_and_trade中获取
                            change_24h: 0.0,
                            fund_type: "telegram".to_string(),
                            timestamp,
                            raw_message: record.raw_message.clone(),
                        };

                        info!("  📨 处理信号: {}", record.symbol);

                        // 所有信号都进入AI分析，不做过滤
                        let trader_clone = trader_for_signals.clone();
                        tokio::spawn(async move {
                            if let Err(e) = trader_clone.analyze_and_trade(alert).await {
                                error!("❌ AI分析交易失败: {}", e);
                            }
                        });

                        // 标记为已处理
                        if let Err(err) = polling_db.mark_telegram_signal_processed(record_id) {
                            error!("❌ 标记信号已处理失败: {}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("❌ 轮询Telegram信号失败: {}", err);
                }
            }
        }
    });
    info!("✅ Telegram信号轮询线程已启动（5秒间隔）\n");

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✅ 所有系统组件已启动完成");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 保持主线程运行
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
