use anyhow::{anyhow, Result};
use dotenv::dotenv;
use grammers_client::{Client, Config, InitParams, Update};
use grammers_session::Session;
use log::{error, info, warn};
use regex::Regex;
use std::env;
use std::time::Duration;
use tokio::time;

use rust_trading_bot::binance_client::BinanceClient;
use rust_trading_bot::exchange_trait::ExchangeClient;
use rust_trading_bot::health_monitor::HealthMonitor;
use rust_trading_bot::trading_lock::TradingLockManager;

#[derive(Debug)]
enum SignalType {
    OpenLong(String),  // 看涨开多
    OpenShort(String), // 看跌开空
    Close(String),     // 跟踪结束平仓
}

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

#[derive(Debug, Clone, Copy)]
enum MultiAssetMode {
    Single,
    Multi,
}

impl MultiAssetMode {
    fn from_env(raw: &str) -> Result<Self> {
        match raw.trim().to_uppercase().as_str() {
            "SINGLE" | "FALSE" | "0" => Ok(Self::Single),
            "MULTI" | "TRUE" | "1" => Ok(Self::Multi),
            other => Err(anyhow!("不支持的 SIGNAL_MULTI_ASSET_MODE 配置: {}", other)),
        }
    }

    fn as_flag(&self) -> bool {
        matches!(self, Self::Multi)
    }

    fn display_label(&self) -> &'static str {
        match self {
            Self::Single => "单币种保证金",
            Self::Multi => "多币种保证金",
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
    // 由于Rust regex不支持负向前瞻，先匹配然后手动检查是否包含"跟踪"
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

async fn execute_signal(
    client: &BinanceClient,
    signal: SignalType,
    leverage: u32,
    margin: f64,
    margin_type: MarginTypeConfig,
    dual_side_position: bool,
    lock_manager: &TradingLockManager,
) -> Result<()> {
    match signal {
        SignalType::OpenLong(symbol) => {
            // 尝试获取开多锁
            if !lock_manager.try_acquire_lock(&symbol, "open_long", "signal_trader", 60)? {
                warn!("⚠️  {} 开多操作被锁定，跳过执行", symbol);
                return Ok(());
            }

            info!(
                "📈 执行开多: {} (杠杆: {}x, 保证金: {}U)",
                symbol, leverage, margin
            );

            let result: Result<()> = async {
                client
                    .ensure_trading_modes(
                        &symbol,
                        leverage,
                        margin_type.as_api_str(),
                        dual_side_position,
                    )
                    .await?;

                // 获取当前价格
                let price = client.get_current_price(&symbol).await?;
                // 获取交易规则
                let trading_rules = client
                    .get_symbol_trading_rules(&symbol)
                    .await
                    .map_err(|e| anyhow::anyhow!("获取{}交易规则失败: {}", symbol, e))?;

                // 正确计算：用指定保证金开杠杆仓位
                let quantity = client
                    .calculate_quantity_with_margin(price, margin, leverage, &trading_rules)
                    .map_err(|e| anyhow::anyhow!("计算数量失败: {}", e))?;

                info!(
                    "💰 交易计算: 保证金{}U × {}倍杠杆 = {}U名义价值",
                    margin,
                    leverage,
                    margin * leverage as f64
                );
                info!("📊 当前价格: {}U, 最终数量: {:.8}", price, quantity);

                client
                    .open_long(
                        &symbol,
                        quantity,
                        leverage,
                        margin_type.as_api_str(),
                        dual_side_position,
                    )
                    .await?;
                info!("✅ 开多成功: {} 数量: {:.8}", symbol, quantity);
                Ok(())
            }
            .await;

            // 释放锁
            lock_manager
                .release_lock(&symbol, "open_long", "signal_trader")
                .ok();
            result?
        }

        SignalType::OpenShort(symbol) => {
            // 尝试获取开空锁
            if !lock_manager.try_acquire_lock(&symbol, "open_short", "signal_trader", 60)? {
                warn!("⚠️  {} 开空操作被锁定，跳过执行", symbol);
                return Ok(());
            }

            info!(
                "📉 执行开空: {} (杠杆: {}x, 保证金: {}U)",
                symbol, leverage, margin
            );

            let result: Result<()> = async {
                client
                    .ensure_trading_modes(
                        &symbol,
                        leverage,
                        margin_type.as_api_str(),
                        dual_side_position,
                    )
                    .await?;

                let price = client.get_current_price(&symbol).await?;
                // 获取交易规则
                let trading_rules = client
                    .get_symbol_trading_rules(&symbol)
                    .await
                    .map_err(|e| anyhow::anyhow!("获取{}交易规则失败: {}", symbol, e))?;

                // 正确计算：用指定保证金开杠杆仓位
                let quantity = client
                    .calculate_quantity_with_margin(price, margin, leverage, &trading_rules)
                    .map_err(|e| anyhow::anyhow!("计算数量失败: {}", e))?;

                info!(
                    "💰 交易计算: 保证金{}U × {}倍杠杆 = {}U名义价值",
                    margin,
                    leverage,
                    margin * leverage as f64
                );
                info!("📊 当前价格: {}U, 最终数量: {:.8}", price, quantity);

                client
                    .open_short(
                        &symbol,
                        quantity,
                        leverage,
                        margin_type.as_api_str(),
                        dual_side_position,
                    )
                    .await?;
                info!("✅ 开空成功: {} 数量: {:.8}", symbol, quantity);
                Ok(())
            }
            .await;

            // 释放锁
            lock_manager
                .release_lock(&symbol, "open_short", "signal_trader")
                .ok();
            result?
        }

        SignalType::Close(symbol) => {
            // 尝试获取平仓锁
            if !lock_manager.try_acquire_lock(&symbol, "close", "signal_trader", 60)? {
                warn!("⚠️  {} 平仓操作被锁定，跳过执行", symbol);
                return Ok(());
            }

            info!("🔄 执行平仓: {}", symbol);

            let result: Result<()> = async {
                // 获取持仓
                let positions = client.get_positions().await?;

                if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
                    client.close_position(&symbol, &pos.side, pos.size).await?;
                    info!("✅ 平仓成功: {} {} {:.4}", symbol, pos.side, pos.size);
                } else {
                    warn!("⚠️  未找到持仓: {}", symbol);
                }
                Ok(())
            }
            .await;

            // 释放锁
            lock_manager
                .release_lock(&symbol, "close", "signal_trader")
                .ok();
            result?
        }
    }

    Ok(())
}

async fn main_loop(
    client: &Client,
    binance_client: &BinanceClient,
    target_channel_id: i64,
    leverage: u32,
    margin: f64,
    margin_type: MarginTypeConfig,
    dual_side_position: bool,
    trading_enabled: bool,
    health_monitor: &HealthMonitor,
    lock_manager: &TradingLockManager,
) -> Result<()> {
    info!("🔍 开始监听目标频道信号...");

    loop {
        // 更新进程状态
        health_monitor
            .update_status("signal_trader", "running")
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
                                if let Err(e) = execute_signal(
                                    binance_client,
                                    signal,
                                    leverage,
                                    margin,
                                    margin_type,
                                    dual_side_position,
                                    lock_manager,
                                )
                                .await
                                {
                                    error!("❌ 执行交易失败: {}", e);
                                }
                            } else {
                                println!("⚠️  交易已禁用，跳过执行");
                            }
                        } else {
                            println!("ℹ️  非交易信号，忽略");
                        }

                        println!();
                    } else {
                        // 静默忽略其他频道的消息，避免日志噪音
                        info!(
                            "🔇 忽略其他频道消息: ID {} (只关注目标频道 {})",
                            channel.id(),
                            target_channel_id
                        );
                    }
                } else {
                    // 静默忽略非频道消息
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

    println!("🤖 Telegram 信号自动交易系统 (带自动重连)\n");

    // 读取配置
    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE")?;
    let target_channel_id: i64 = env::var("TARGET_CHANNEL_ID")?.parse()?;

    let binance_api_key = env::var("BINANCE_API_KEY")?;
    let binance_secret = env::var("BINANCE_SECRET_KEY")?;
    let testnet = env::var("BINANCE_TESTNET")?.parse::<bool>()?;

    let leverage: u32 = env::var("SIGNAL_LEVERAGE")?.parse()?;
    let margin: f64 = env::var("SIGNAL_MARGIN")?.parse()?;
    let margin_type_raw = env::var("SIGNAL_MARGIN_TYPE").unwrap_or_else(|_| "CROSSED".to_string());
    let margin_type = MarginTypeConfig::from_env(&margin_type_raw)?;
    let multi_asset_mode_raw =
        env::var("SIGNAL_MULTI_ASSET_MODE").unwrap_or_else(|_| "SINGLE".to_string());
    let multi_asset_mode = MultiAssetMode::from_env(&multi_asset_mode_raw)?;
    let position_mode_raw =
        env::var("SIGNAL_POSITION_MODE").unwrap_or_else(|_| "SINGLE".to_string());
    let dual_side_position = matches!(position_mode_raw.trim().to_uppercase().as_str(), "DUAL");
    let trading_enabled = env::var("SIGNAL_TRADING_ENABLED")?.parse::<bool>()?;

    println!("🔑 Telegram API ID: {}", api_id);
    println!("📱 手机号: {}", phone);
    println!("🎯 监听频道 ID: {}", target_channel_id);
    println!(
        "💰 Binance 主网: {}",
        if testnet { "测试网" } else { "主网" }
    );
    println!("⚡ 杠杆: {}x", leverage);
    println!("💵 保证金: {} USDT", margin);
    println!("🏦 仓位模式: {}", margin_type.display_label());
    println!("💱 保证金资产模式: {}", multi_asset_mode.display_label());
    println!(
        "📐 持仓模式: {}",
        if dual_side_position {
            "双向持仓"
        } else {
            "单向持仓"
        }
    );
    println!(
        "🔄 交易状态: {}\n",
        if trading_enabled {
            "✅ 启用"
        } else {
            "❌ 禁用 (仅监听)"
        }
    );

    if !trading_enabled {
        println!("⚠️  交易功能已禁用，仅监听和解析信号");
        println!("⚠️  启用交易: 设置 SIGNAL_TRADING_ENABLED=true\n");
    }

    // 连接 Binance
    let binance_client = BinanceClient::new(binance_api_key, binance_secret, testnet);
    println!("✅ Binance 客户端初始化完成");
    binance_client
        .set_multi_assets_margin(multi_asset_mode.as_flag())
        .await?;

    // 初始化健康监控
    let health_monitor = HealthMonitor::new();
    health_monitor
        .update_status("signal_trader", "starting")
        .ok();

    // 初始化交易锁管理器
    let lock_manager = TradingLockManager::new();
    lock_manager.cleanup_expired_locks().ok();

    println!("\n🔄 启动带自动重连的Telegram监听系统...");
    println!("════════════════════════════════════════");
    println!("🎯 只监控频道: {} (CM AI SIGNAL)", target_channel_id);
    println!("🔄 自动重连: 启用退避策略");
    println!("🚫 忽略所有其他频道，减少Gap影响");
    println!("════════════════════════════════════════\n");

    // 🔥 关键的自动重连循环 🔥
    let mut reconnect_delay = Duration::from_secs(1);
    let max_reconnect_delay = Duration::from_secs(60);

    loop {
        println!("🔄 连接到 Telegram...");

        // 尝试连接，使用官方客户端参数避免检测
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

                // 检查授权状态
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

                // 重置重连延迟
                reconnect_delay = Duration::from_secs(1);
                client
            }
            Err(e) => {
                error!("❌ 连接失败: {}", e);
                println!("🔄 等待 {:?} 后重试连接...", reconnect_delay);
                time::sleep(reconnect_delay).await;

                // 增加重连延迟，避免因频繁重连被 Telegram 限制
                reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
                continue;
            }
        };

        // 获取目标频道对象
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
            return Err(anyhow::anyhow!(
                "❌ 未找到目标频道 ID: {}",
                target_channel_id
            ));
        }

        println!("🔄 运行客户端...");

        // 运行客户端消息处理循环
        if let Err(e) = main_loop(
            &client,
            &binance_client,
            target_channel_id,
            leverage,
            margin,
            margin_type,
            dual_side_position,
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

        // 增加重连延迟，避免因频繁重连被 Telegram 限制
        reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
    }
}
