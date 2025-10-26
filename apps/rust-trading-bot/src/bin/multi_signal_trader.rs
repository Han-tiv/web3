// 多交易所并发信号交易系统
use anyhow::{anyhow, Result};
use dotenv::dotenv;
use grammers_client::{Client, Config, InitParams, Update};
use grammers_session::Session;
use log::{error, info, warn};
use regex::Regex;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use rust_trading_bot::bitget_client::BitgetClient;
use rust_trading_bot::bybit_client::BybitClient;
use rust_trading_bot::exchange_trait::ExchangeClient;
use rust_trading_bot::gate_client::GateClient;
use rust_trading_bot::health_monitor::HealthMonitor;
use rust_trading_bot::hyperliquid_client::HyperliquidClient;
use rust_trading_bot::multi_exchange_executor::{MultiExchangeExecutor, SignalType};
use rust_trading_bot::okx_client::OkxClient;
use rust_trading_bot::trading_lock::TradingLockManager;

#[derive(Debug, Clone, Copy)]
enum MarginTypeConfig {
    Crossed,
    Isolated,
}

impl MarginTypeConfig {
    fn from_env(raw: &str) -> Result<Self> {
        match raw.trim().to_uppercase().as_str() {
            "CROSSED" => Ok(Self::Crossed),
            "ISOLATED" => Ok(Self::Isolated),
            other => Err(anyhow!("不支持的 SIGNAL_MARGIN_TYPE 配置: {}", other)),
        }
    }

    fn as_api_str(&self) -> &'static str {
        match self {
            Self::Crossed => "CROSSED",
            Self::Isolated => "ISOLATED",
        }
    }

    fn display_label(&self) -> &'static str {
        match self {
            Self::Crossed => "全仓模式",
            Self::Isolated => "逐仓模式",
        }
    }
}

fn parse_signal(text: &str) -> Option<SignalType> {
    // 优先匹配平仓信号: SUPERUSDT - 看跌跟踪结束 或 看涨跟踪结束
    let close_re = Regex::new(r"(\w+USDT)\s*-\s*看(?:跌|涨)跟踪结束").ok()?;
    if let Some(caps) = close_re.captures(text) {
        let symbol = caps.get(1)?.as_str().to_string();
        return Some(SignalType::Close(symbol));
    }

    // 然后匹配开仓信号: B2USDT - 看跌📉 或 B2USDT - 看涨📈
    let open_re = Regex::new(r"(\w+USDT)\s*-\s*看(跌|涨)").ok()?;
    if let Some(caps) = open_re.captures(text) {
        // 如果包含"跟踪"字符，跳过（这不是开仓信号）
        if text.contains("跟踪") {
            return None;
        }

        let symbol = caps.get(1)?.as_str().to_string();
        let direction = caps.get(2)?.as_str();

        return match direction {
            "涨" => Some(SignalType::OpenLong(symbol)),
            "跌" => Some(SignalType::OpenShort(symbol)),
            _ => None,
        };
    }

    None
}

async fn main_loop(
    client: &Client,
    executor: &MultiExchangeExecutor,
    target_channel_id: i64,
    trading_enabled: bool,
    health_monitor: &HealthMonitor,
    lock_manager: &TradingLockManager,
) -> Result<()> {
    info!("🔍 开始监听目标频道信号...\n");

    loop {
        // 更新进程状态
        health_monitor
            .update_status("multi_signal_trader", "running")
            .ok();

        match client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => {
                let chat = message.chat();

                if let grammers_client::types::Chat::Channel(channel) = chat {
                    if channel.id() == target_channel_id {
                        let text = message.text();

                        println!(
                            "📨 [{}] 目标频道新消息",
                            chrono::Utc::now().format("%H:%M:%S")
                        );
                        println!("   频道ID: {} (CM AI SIGNAL)", target_channel_id);
                        println!("   完整内容: {}", text);

                        // 解析信号
                        if let Some(signal) = parse_signal(text) {
                            println!("🎯 检测到信号: {:?}", signal);
                            println!("   消息内容:\n{}", text);
                            println!("   ────────────────────────────────────");

                            if trading_enabled {
                                // 获取信号对应的交易对
                                let symbol = match &signal {
                                    SignalType::OpenLong(s) | SignalType::OpenShort(s) | SignalType::Close(s) => s,
                                };

                                // 获取锁类型
                                let lock_type = match &signal {
                                    SignalType::OpenLong(_) => "open_long",
                                    SignalType::OpenShort(_) => "open_short",
                                    SignalType::Close(_) => "close",
                                };

                                // 尝试获取锁
                                if !lock_manager.try_acquire_lock(symbol, lock_type, "multi_signal_trader", 60)? {
                                    warn!("⚠️  {} {} 操作被锁定，跳过执行", symbol, lock_type);
                                    continue;
                                }

                                info!("🚀 开始并发执行到所有交易所...\n");
                                
                                // 并发执行到所有交易所
                                let results = executor.execute_signal(signal.clone()).await;

                                // 释放锁
                                lock_manager.release_lock(symbol, lock_type, "multi_signal_trader").ok();

                                // 打印结果
                                println!("\n📊 执行结果汇总:");
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                let mut success_count = 0;
                                let mut fail_count = 0;

                                for result in results {
                                    match result {
                                        Ok(msg) => {
                                            println!("✅ {}", msg);
                                            success_count += 1;
                                        }
                                        Err(e) => {
                                            println!("❌ 执行失败: {}", e);
                                            fail_count += 1;
                                        }
                                    }
                                }

                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                println!("✅ 成功: {} | ❌ 失败: {}", success_count, fail_count);
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                                // 打印账户和持仓摘要
                                executor.print_accounts_summary().await;
                                executor.print_positions_summary().await;
                            } else {
                                println!("⚠️  交易已禁用，跳过执行");
                            }
                        } else {
                            println!("ℹ️  非交易信号，忽略");
                        }

                        println!();
                    } else {
                        info!(
                            "🔇 忽略其他频道消息: ID {} (只关注目标频道 {})",
                            channel.id(),
                            target_channel_id
                        );
                    }
                } else {
                    info!("🔇 忽略非频道消息");
                }
            }
            Ok(_) => {
                // 其他类型的更新，继续监听
            }
            Err(e) => {
                error!("❌ Telegram连接错误: {}", e);
                return Err(e.into());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("🤖 多交易所并发信号交易系统\n");

    // 读取 Telegram 配置
    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE")?;
    let target_channel_id: i64 = env::var("TARGET_CHANNEL_ID")?.parse()?;

    // 读取交易配置
    let leverage: u32 = env::var("SIGNAL_LEVERAGE")?.parse()?;
    let margin: f64 = env::var("SIGNAL_MARGIN")?.parse()?;
    let margin_type_raw = env::var("SIGNAL_MARGIN_TYPE").unwrap_or_else(|_| "CROSSED".to_string());
    let margin_type = MarginTypeConfig::from_env(&margin_type_raw)?;
    let position_mode_raw = env::var("SIGNAL_POSITION_MODE").unwrap_or_else(|_| "SINGLE".to_string());
    let dual_side_position = matches!(position_mode_raw.trim().to_uppercase().as_str(), "DUAL");
    let trading_enabled = env::var("SIGNAL_TRADING_ENABLED")?.parse::<bool>()?;

    // 创建所有交易所客户端
    let mut exchanges: Vec<Arc<dyn ExchangeClient>> = Vec::new();

    // Binance
    if let (Ok(key), Ok(secret)) = (env::var("BINANCE_API_KEY"), env::var("BINANCE_SECRET")) {
        let testnet = env::var("BINANCE_TESTNET").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);
        let client = rust_trading_bot::binance_client::BinanceClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ Binance 客户端已加载");
    }

    // OKX
    if let (Ok(key), Ok(secret), Ok(passphrase)) = (
        env::var("OKX_API_KEY"),
        env::var("OKX_SECRET"),
        env::var("OKX_PASSPHRASE"),
    ) {
        let testnet = env::var("OKX_TESTNET").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);
        let client = OkxClient::new(key, secret, passphrase, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ OKX 客户端已加载");
    }

    // Bitget
    if let (Ok(key), Ok(secret), Ok(passphrase)) = (
        env::var("BITGET_API_KEY"),
        env::var("BITGET_SECRET"),
        env::var("BITGET_PASSPHRASE"),
    ) {
        let testnet = env::var("BITGET_TESTNET").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);
        let client = BitgetClient::new(key, secret, passphrase, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ Bitget 客户端已加载");
    }

    // Bybit
    if let (Ok(key), Ok(secret)) = (env::var("BYBIT_API_KEY"), env::var("BYBIT_SECRET")) {
        let testnet = env::var("BYBIT_TESTNET").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);
        let client = BybitClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ Bybit 客户端已加载");
    }

    // Gate
    if let (Ok(key), Ok(secret)) = (env::var("GATE_API_KEY"), env::var("GATE_SECRET")) {
        let testnet = env::var("GATE_TESTNET").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false);
        let client = GateClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ Gate 客户端已加载");
    }

    // Hyperliquid (完整交易功能)
    if let (Ok(address), Ok(secret)) = (env::var("HYPERLIQUID_ADDRESS"), env::var("HYPERLIQUID_SECRET")) {
        let proxy_address = env::var("HYPERLIQUID_PROXY_ADDRESS").unwrap_or_else(|_| "".to_string());
        let testnet = env::var("HYPERLIQUID_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = HyperliquidClient::new(address, proxy_address, secret, testnet);
        exchanges.push(Arc::new(client));
        info!("✅ Hyperliquid 客户端已加载");
    }

    if exchanges.is_empty() {
        return Err(anyhow!("❌ 未配置任何交易所 API"));
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 配置摘要");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔑 Telegram API ID: {}", api_id);
    println!("📱 手机号: {}", phone);
    println!("🎯 监听频道 ID: {}", target_channel_id);
    println!("⚡ 杠杆: {}x", leverage);
    println!("💵 保证金: {} USDT", margin);
    println!("🏦 仓位模式: {}", margin_type.display_label());
    println!(
        "📐 持仓模式: {}",
        if dual_side_position { "双向持仓" } else { "单向持仓" }
    );
    println!("🏢 已加载交易所数量: {}", exchanges.len());
    for exchange in &exchanges {
        println!("   ✓ {}", exchange.get_exchange_name());
    }
    println!(
        "🔄 交易状态: {}",
        if trading_enabled { "✅ 启用" } else { "❌ 禁用 (仅监听)" }
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if !trading_enabled {
        println!("⚠️  交易功能已禁用，仅监听和解析信号");
        println!("⚠️  启用交易: 设置 SIGNAL_TRADING_ENABLED=true\n");
    }

    // 创建多交易所执行器
    let executor = MultiExchangeExecutor::new(
        exchanges,
        leverage,
        margin,
        margin_type.as_api_str().to_string(),
        dual_side_position,
    );

    // 初始化健康监控
    let health_monitor = HealthMonitor::new();
    health_monitor.update_status("multi_signal_trader", "starting").ok();

    // 初始化交易锁管理器
    let lock_manager = TradingLockManager::new();
    lock_manager.cleanup_expired_locks().ok();

    // 打印初始账户摘要
    info!("📊 获取初始账户信息...\n");
    executor.print_accounts_summary().await;
    executor.print_positions_summary().await;

    println!("\n🔄 启动带自动重连的Telegram监听系统...");
    println!("════════════════════════════════════════");
    println!("🎯 只监控频道: {} (CM AI SIGNAL)", target_channel_id);
    println!("🔄 自动重连: 启用退避策略");
    println!("🚫 忽略所有其他频道，减少Gap影响");
    println!("════════════════════════════════════════\n");

    // 自动重连循环
    let mut reconnect_delay = Duration::from_secs(1);
    let max_reconnect_delay = Duration::from_secs(60);

    loop {
        println!("🔄 连接到 Telegram...");

        let client_result = Client::connect(Config {
            session: Session::load_file_or_create("session.session")?,
            api_id,
            api_hash: api_hash.clone(),
            params: InitParams {
                device_model: "Desktop".to_string(),
                system_version: "Windows 10".to_string(),
                app_version: "5.12.3 x64".to_string(),
                lang_code: "en".to_string(),
                system_lang_code: "en-US".to_string(),
                catch_up: true,
                ..Default::default()
            },
        })
        .await;

        let client = match client_result {
            Ok(client) => {
                println!("✅ Telegram 连接成功");

                if !client.is_authorized().await? {
                    println!("⚠️  需要登录");
                    println!("📨 发送验证码到 {}...", phone);

                    let token = client.request_login_code(&phone).await?;
                    println!("✅ 验证码已发送");

                    println!("\n🔢 请输入收到的验证码:");
                    let mut code = String::new();
                    std::io::stdin().read_line(&mut code)?;
                    let code = code.trim();

                    client.sign_in(&token, code).await?;
                    println!("✅ 登录成功!");
                    client.session().save_to_file("session.session")?;
                } else {
                    println!("✅ 已登录");
                }

                reconnect_delay = Duration::from_secs(1);
                client
            }
            Err(e) => {
                error!("❌ 连接失败: {}", e);
                println!("🔄 等待 {:?} 后重试连接...", reconnect_delay);
                time::sleep(reconnect_delay).await;

                reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
                continue;
            }
        };

        // 获取目标频道
        println!("🔍 获取目标频道信息...");
        let mut dialogs = client.iter_dialogs();
        let mut target_channel = None;

        while let Some(dialog) = dialogs.next().await? {
            if let grammers_client::types::Chat::Channel(channel) = dialog.chat() {
                if channel.id() == target_channel_id {
                    target_channel = Some(channel.clone());
                    println!(
                        "✅ 找到目标频道: {} (ID: {})",
                        channel.title(),
                        channel.id()
                    );
                    break;
                }
            }
        }

        if target_channel.is_none() {
            return Err(anyhow!("❌ 未找到目标频道 ID: {}", target_channel_id));
        }

        println!("🔄 运行客户端...\n");

        // 运行消息处理循环
        if let Err(e) = main_loop(
            &client,
            &executor,
            target_channel_id,
            trading_enabled,
            &health_monitor,
            &lock_manager,
        )
        .await
        {
            error!("📡 消息处理循环错误: {}", e);
        }

        println!("🔌 客户端断开连接. 等待 {:?} 后重连...", reconnect_delay);
        time::sleep(reconnect_delay).await;

        reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
    }
}
