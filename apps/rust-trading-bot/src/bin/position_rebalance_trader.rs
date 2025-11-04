//! 定时仓位重新评估系统
//!
//! 线程1：监听 Telegram 频道 -> CoinParser 解析 -> 信号入队
//! 线程2：每3分钟周期 -> 聚合信号 + 持仓 -> AI 批量分析 -> 仓位协调 -> 交易执行 -> 状态同步
//!
//! 设计目标：复用既有模块（SignalManager, CoinParser, AiDecisionEngine, PositionCoordinator, TradeExecutor）
//! 并保持日志可观测性，同时避免自研组件。

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use dotenv::dotenv;
use grammers_client::{Client, Config, Update};
use grammers_session::Session;
use log::{error, info, warn};
use tokio::task::JoinSet;

use rust_trading_bot::ai_decision_engine::{AiDecisionConfig, AiDecisionEngine, CoinInfo};
use rust_trading_bot::binance_client::BinanceClient;
use rust_trading_bot::coin_parser::CoinParser;
use rust_trading_bot::deepseek_client::DeepSeekClient;
use rust_trading_bot::exchange_trait::{ExchangeClient, Position};
use rust_trading_bot::position_coordinator::{PositionCoordinator, PositionCoordinatorConfig};
use rust_trading_bot::signal_manager::{SignalManager, SignalManagerConfig, SignalSource};
use rust_trading_bot::technical_analysis::TechnicalAnalyzer;
use rust_trading_bot::trade_executor::{TradeExecutor, TradeExecutorConfig};

/// 用于集中存放所有运行所需环境配置
#[derive(Clone)]
struct AppConfig {
    telegram_api_id: i32,
    telegram_api_hash: String,
    telegram_session_path: String,
    telegram_channel_id: i64,
    telegram_channel_name: String,
    rebalance_interval_secs: u64,
    signal_window_secs: i64,
    signal_ttl_secs: i64,
    deepseek_api_key: String,
    binance_api_key: String,
    binance_secret: String,
    binance_testnet: bool,
    leverage: u32,
    base_position_usdt: f64,
    max_position_usdt: f64,
    min_api_interval_ms: u64,
    margin_type: String,
    dual_side_position: bool,
    parser_strict_mode: bool,
    max_ai_concurrency: usize,
    ai_call_timeout_secs: u64,
    cooldown_period_secs: i64,
    max_adjustments_per_cycle: usize,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        let telegram_api_id = env_or_parse::<i32>("TELEGRAM_API_ID", None)
            .context("缺少 TELEGRAM_API_ID (Telegram 应用 ID)")?;
        let telegram_api_hash =
            env::var("TELEGRAM_API_HASH").context("缺少 TELEGRAM_API_HASH (Telegram 应用 Hash)")?;
        let telegram_session_path =
            env::var("TELEGRAM_SESSION_PATH").unwrap_or_else(|_| "session.session".to_string());

        let telegram_channel_id =
            env_or_parse::<i64>("TELEGRAM_SIGNAL_CHANNEL_ID", Some(2254462672_i64))?;
        let telegram_channel_name =
            env::var("TELEGRAM_SIGNAL_CHANNEL_NAME").unwrap_or_else(|_| "Valuescan".to_string());

        let rebalance_interval_secs = env_or_parse::<u64>("REBALANCE_INTERVAL_SECS", Some(180))?;
        let signal_window_secs = env_or_parse::<i64>("SIGNAL_WINDOW_SECS", Some(180))?;
        let signal_ttl_secs = env_or_parse::<i64>("SIGNAL_TTL_SECS", Some(600))?;

        let deepseek_api_key =
            env::var("DEEPSEEK_API_KEY").context("缺少 DEEPSEEK_API_KEY (DeepSeek API Key)")?;

        let binance_api_key = env::var("BINANCE_API_KEY").context("缺少 BINANCE_API_KEY")?;
        let binance_secret = env::var("BINANCE_SECRET").context("缺少 BINANCE_SECRET")?;
        let binance_testnet = env_or_bool("BINANCE_TESTNET", Some(false))?;

        let leverage = env_or_parse::<u32>("TRADE_LEVERAGE", Some(5))?;
        let base_position_usdt = env_or_parse::<f64>("TRADE_BASE_POSITION_USDT", Some(6.0))?;
        let max_position_usdt = env_or_parse::<f64>("TRADE_MAX_POSITION_USDT", Some(100.0))?;
        let min_api_interval_ms = env_or_parse::<u64>("TRADE_MIN_API_INTERVAL_MS", Some(500))?;
        let margin_type = env::var("TRADE_MARGIN_TYPE").unwrap_or_else(|_| "cross".to_string());
        let dual_side_position = env_or_bool("TRADE_DUAL_SIDE", Some(false))?;
        let parser_strict_mode = env_or_bool("PARSE_STRICT_MODE", Some(true))?;

        let max_ai_concurrency = env_or_parse::<usize>("AI_MAX_CONCURRENCY", Some(5))?;
        let ai_call_timeout_secs = env_or_parse::<u64>("AI_CALL_TIMEOUT_SECS", Some(10))?;

        let cooldown_period_secs = env_or_parse::<i64>("POSITION_COOLDOWN_SECS", Some(300))?;
        let max_adjustments_per_cycle = env_or_parse::<usize>("POSITION_MAX_ADJUSTMENTS", Some(2))?;

        Ok(Self {
            telegram_api_id,
            telegram_api_hash,
            telegram_session_path,
            telegram_channel_id,
            telegram_channel_name,
            rebalance_interval_secs,
            signal_window_secs,
            signal_ttl_secs,
            deepseek_api_key,
            binance_api_key,
            binance_secret,
            binance_testnet,
            leverage,
            base_position_usdt,
            max_position_usdt,
            min_api_interval_ms,
            margin_type,
            dual_side_position,
            parser_strict_mode,
            max_ai_concurrency,
            ai_call_timeout_secs,
            cooldown_period_secs,
            max_adjustments_per_cycle,
        })
    }
}

fn env_or_parse<T: std::str::FromStr>(key: &str, default: Option<T>) -> Result<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match std::env::var(key) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("环境变量 {} 格式错误: {} - {}", key, val, e)),
        Err(std::env::VarError::NotPresent) => {
            default.ok_or_else(|| anyhow::anyhow!(format!("缺少必需环境变量 {}", key)))
        }
        Err(e) => Err(anyhow::anyhow!(format!("读取环境变量 {} 失败: {}", key, e))),
    }
}

fn env_or_bool(key: &str, default: Option<bool>) -> Result<bool> {
    match std::env::var(key) {
        Ok(val) => match val.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" => Ok(false),
            _ => bail!(
                "环境变量 {} 仅支持 true/false/1/0/yes/no，当前为 {}",
                key,
                val
            ),
        },
        Err(std::env::VarError::NotPresent) => {
            default.ok_or_else(|| anyhow::anyhow!(format!("缺少必需环境变量 {}", key)))
        }
        Err(e) => Err(anyhow::anyhow!(format!("读取环境变量 {} 失败: {}", key, e))),
    }
}

/// 线程之间共享的状态与依赖
#[derive(Clone)]
struct RuntimeContext {
    config: Arc<AppConfig>,
    signal_manager: Arc<SignalManager>,
    coin_parser: Arc<CoinParser>,
    position_coordinator: Arc<PositionCoordinator>,
    ai_engine: Arc<AiDecisionEngine>,
    trade_executor: Arc<TradeExecutor>,
    exchange: Arc<dyn ExchangeClient>,
}

/// 应用入口：初始化依赖并启动双线程管线
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 Position Rebalance Trader 启动");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let config = Arc::new(AppConfig::from_env()?);

    info!(
        "📡 Telegram 目标频道: {} ({})",
        config.telegram_channel_name, config.telegram_channel_id
    );
    info!(
        "⏱️ 周期: {} 秒 | 信号窗口: {} 秒",
        config.rebalance_interval_secs, config.signal_window_secs
    );
    info!(
        "⚙️ 杠杆: {}x | 基础仓位: {} USDT | 最大仓位: {} USDT",
        config.leverage, config.base_position_usdt, config.max_position_usdt
    );

    let telegram_client = Client::connect(Config {
        session: Session::load_file_or_create(&config.telegram_session_path)?,
        api_id: config.telegram_api_id,
        api_hash: config.telegram_api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    if !telegram_client.is_authorized().await? {
        bail!("❌ Telegram 未授权，请先运行 `cargo run --bin get_channels` 完成登录");
    }

    info!("✅ Telegram 已连接");

    let exchange_client = Arc::new(BinanceClient::new(
        config.binance_api_key.clone(),
        config.binance_secret.clone(),
        config.binance_testnet,
    ));
    let exchange_trait: Arc<dyn ExchangeClient> = exchange_client.clone();

    let signal_manager = Arc::new(SignalManager::new(SignalManagerConfig {
        dedup_window_secs: config.signal_window_secs,
        signal_ttl_secs: config.signal_ttl_secs,
        ..Default::default()
    }));
    signal_manager.clone().start_cleanup_task();

    let coin_parser = Arc::new(CoinParser::new(config.parser_strict_mode));
    let deepseek_client = Arc::new(DeepSeekClient::new(config.deepseek_api_key.clone()));
    let analyzer = Arc::new(TechnicalAnalyzer::new());
    let ai_engine = Arc::new(AiDecisionEngine::new(
        AiDecisionConfig {
            max_concurrent_calls: config.max_ai_concurrency,
            call_timeout_secs: config.ai_call_timeout_secs,
            ..Default::default()
        },
        deepseek_client,
        analyzer,
    ));

    let position_coordinator = Arc::new(PositionCoordinator::new(PositionCoordinatorConfig {
        cooldown_period_secs: config.cooldown_period_secs,
        max_adjustments_per_cycle: config.max_adjustments_per_cycle,
        ..Default::default()
    }));

    let trade_executor = Arc::new(TradeExecutor::new(
        TradeExecutorConfig {
            min_api_interval_ms: config.min_api_interval_ms,
            max_position_usdt: config.max_position_usdt,
            base_position_usdt: config.base_position_usdt,
            margin_type: config.margin_type.clone(),
            dual_side_position: config.dual_side_position,
        },
        exchange_trait.clone(),
    ));

    let runtime = RuntimeContext {
        config: config.clone(),
        signal_manager: signal_manager.clone(),
        coin_parser: coin_parser.clone(),
        position_coordinator: position_coordinator.clone(),
        ai_engine: ai_engine.clone(),
        trade_executor: trade_executor.clone(),
        exchange: exchange_trait.clone(),
    };

    info!(
        "🧠 DeepSeek AI 已配置，最大并发 {}，超时 {} 秒",
        config.max_ai_concurrency, config.ai_call_timeout_secs
    );
    info!(
        "🏦 Binance 模式: {}",
        if config.binance_testnet {
            "Testnet"
        } else {
            "Futures"
        }
    );

    let mut tasks = JoinSet::new();
    let telegram_arc = Arc::new(telegram_client);
    let signal_source = SignalSource::Channel {
        id: config.telegram_channel_id,
        name: config.telegram_channel_name.clone(),
    };

    // 线程1：Telegram 监听
    tasks.spawn(run_telegram_listener(
        telegram_arc.clone(),
        runtime.signal_manager.clone(),
        runtime.coin_parser.clone(),
        signal_source.clone(),
    ));

    // 线程2：定时重新评估
    tasks.spawn(run_rebalance_loop(runtime.clone()));

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(_)) => { /* 单个任务正常结束，继续等待其他任务 */ }
            Ok(Err(e)) => {
                error!("工作线程异常: {}", e);
                break;
            }
            Err(join_err) => {
                error!("任务 Join 失败: {}", join_err);
                break;
            }
        }
    }

    Ok(())
}

/// 监听 Telegram 频道，解析消息并异步写入信号队列
async fn run_telegram_listener(
    client: Arc<Client>,
    signal_manager: Arc<SignalManager>,
    coin_parser: Arc<CoinParser>,
    signal_source: SignalSource,
) -> Result<()> {
    info!("📨 Telegram 监听线程已启动");

    let (target_channel_id, target_channel_name) = match &signal_source {
        SignalSource::Channel { id, name } => (*id, name.clone()),
        other => {
            warn!("⚠️ 不支持的信号来源: {:?}，监听线程退出", other);
            return Ok(());
        }
    };

    loop {
        match client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => {
                let chat = message.chat();
                if chat.id() == target_channel_id {
                    let text = message.text();
                    if text.is_empty() {
                        continue;
                    }

                    info!(
                        "🆕 频道消息 [{} - {}] @ {}: {}",
                        target_channel_name,
                        target_channel_id,
                        Utc::now().format("%H:%M:%S"),
                        text
                    );
                    let signals = coin_parser.parse_to_signal(text, signal_source.clone());

                    if signals.is_empty() {
                        info!("ℹ️ 消息未解析出有效币种，跳过");
                        continue;
                    }

                    let added = signal_manager.add_signals(signals).await;
                    info!("✅ 信号入队成功: {} 条", added);
                }
            }
            Ok(_) => { /* 其他更新类型忽略 */ }
            Err(e) => {
                error!("Telegram 监听错误: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// 定时触发仓位重新评估的主循环
async fn run_rebalance_loop(runtime: RuntimeContext) -> Result<()> {
    info!(
        "⏳ 定时评估线程已启动，间隔 {} 秒",
        runtime.config.rebalance_interval_secs
    );
    let mut interval =
        tokio::time::interval(Duration::from_secs(runtime.config.rebalance_interval_secs));

    // 立即等待首个 tick，确保间隔一致
    interval.tick().await;

    loop {
        interval.tick().await;
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🕒 开始新一轮仓位评估 @ {}", Utc::now());

        if let Err(e) = process_rebalance_cycle(&runtime).await {
            error!("❌ 评估周期失败: {}", e);
        }
    }
}

/// 执行一轮完整的信号收集、AI 分析、交易执行与状态同步
async fn process_rebalance_cycle(runtime: &RuntimeContext) -> Result<()> {
    let config = &runtime.config;

    runtime.position_coordinator.reset_cycle_counters().await;

    let exchange_positions = runtime
        .exchange
        .get_positions()
        .await
        .context("获取交易所持仓失败")?;
    runtime
        .position_coordinator
        .sync_positions(exchange_positions.clone())
        .await;

    let recent_signals = runtime
        .signal_manager
        .drain_recent(config.signal_window_secs)
        .await;

    if recent_signals.is_empty() {
        info!("📭 最近 {} 秒无新增信号", config.signal_window_secs);
    } else {
        info!("📥 收集最近信号 {} 条", recent_signals.len());
    }

    let mut symbols: HashSet<String> = SignalManager::dedup_symbols(&recent_signals)
        .into_iter()
        .collect();
    for position in &exchange_positions {
        symbols.insert(position.symbol.clone());
    }

    if symbols.is_empty() {
        info!("🚫 无需分析：无信号且无持仓");
        return Ok(());
    }

    let position_lookup: HashMap<String, Position> = exchange_positions
        .into_iter()
        .map(|pos| (pos.symbol.clone(), pos))
        .collect();

    let coin_infos: Vec<CoinInfo> = symbols
        .iter()
        .map(|symbol| CoinInfo {
            symbol: symbol.clone(),
            current_position: position_lookup.get(symbol).cloned(),
        })
        .collect();

    info!("🧠 AI 批量分析 {} 个币种", coin_infos.len());
    let decisions = runtime
        .ai_engine
        .analyze_batch(coin_infos, runtime.exchange.clone())
        .await;

    if decisions.is_empty() {
        info!("ℹ️ AI 未返回有效决策，结束本轮");
        return Ok(());
    }

    info!("🗂️ 仓位协调处理 {} 个决策", decisions.len());
    let actions = runtime
        .position_coordinator
        .merge_decisions_to_plan(decisions, config.leverage)
        .await;

    if actions.is_empty() {
        info!("✅ 无需调整仓位");
        return Ok(());
    }

    info!("🛠️ 执行交易动作 {} 个", actions.len());
    let (results, stats) = runtime.trade_executor.execute_plan(actions).await;

    info!(
        "📊 执行统计: 成功 {} / 失败 {}",
        stats.successful, stats.failed
    );

    let affected_symbols: HashSet<String> = results
        .iter()
        .filter(|res| res.success)
        .map(|res| res.symbol.clone())
        .collect();

    if !affected_symbols.is_empty() {
        let latest_positions = runtime
            .exchange
            .get_positions()
            .await
            .context("交易后获取最新持仓失败")?;

        let latest_lookup: HashMap<String, Position> = latest_positions
            .into_iter()
            .map(|pos| (pos.symbol.clone(), pos))
            .collect();

        for symbol in affected_symbols {
            let maybe_position = latest_lookup.get(&symbol).cloned();
            runtime
                .position_coordinator
                .update_position_after_trade(&symbol, maybe_position)
                .await;
        }
    }

    info!("✅ 本轮评估结束\n");
    Ok(())
}
