use anyhow::{anyhow, Result};
use chrono::Utc;
use dotenv::dotenv;
use log::{error, info, warn};
use reqwest::Client;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use tokio::time::{sleep, Duration};

use rust_trading_bot::binance_client::BinanceClient;
use rust_trading_bot::exchange_trait::ExchangeClient;
use rust_trading_bot::health_monitor::HealthMonitor;
use rust_trading_bot::trading_lock::TradingLockManager;

/// 计算持仓回报率
fn calculate_profit_rate(entry_price: f64, current_price: f64, side: &str, leverage: u32) -> f64 {
    let price_change_rate = match side {
        "LONG" => (current_price - entry_price) / entry_price,
        "SHORT" => (entry_price - current_price) / entry_price,
        _ => 0.0,
    };

    price_change_rate * leverage as f64
}

#[derive(Clone)]
struct TelegramConfig {
    bot_token: String,
    chat_id: String,
}

async fn send_telegram_alert(
    client: &Client,
    config: &TelegramConfig,
    message: &str,
) -> Result<()> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );

    let payload = json!({
        "chat_id": config.chat_id,
        "text": message,
        "parse_mode": "HTML"
    });

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|err| anyhow!("发送Telegram消息失败: {}", err))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Telegram返回错误: {} - {}", status, body));
    }

    Ok(())
}

async fn monitor_positions(
    client: &BinanceClient,
    stop_loss_percent: f64,
    alert_percent: f64,
    leverage: u32,
    health_monitor: &HealthMonitor,
    lock_manager: &TradingLockManager,
    telegram_client: &Client,
    telegram_config: &TelegramConfig,
    auto_close_enabled: bool,
) -> Result<()> {
    let mut alerted_positions: HashSet<String> = HashSet::new();

    loop {
        // 更新进程状态
        health_monitor
            .update_status("profit_monitor", "running")
            .ok();

        // 检查signal_trader是否健康
        if !health_monitor.is_process_healthy("signal_trader", 300) {
            // 5分钟超时
            warn!("⚠️  Signal Trader进程异常，请检查");
        }

        match client.get_positions().await {
            Ok(positions) => {
                let active_positions: Vec<_> = positions
                    .iter()
                    .filter(|pos| pos.size.abs() > 0.0)
                    .collect();

                if active_positions.is_empty() {
                    info!("📊 当前无持仓，继续监控...");
                } else {
                    info!("📊 监控 {} 个持仓的回报率:", active_positions.len());

                    let mut current_position_keys: HashSet<String> = HashSet::new();

                    for pos in &active_positions {
                        // 获取当前市价
                        match client.get_current_price(&pos.symbol).await {
                            Ok(current_price) => {
                                let profit_rate = calculate_profit_rate(
                                    pos.entry_price,
                                    current_price,
                                    &pos.side,
                                    leverage, // 使用配置的杠杆而不是硬编码
                                );

                                let profit_percent = profit_rate * 100.0;

                                info!(
                                    "   {} {}: 入场${:.4} 当前${:.4} 回报率{:.2}%",
                                    pos.symbol,
                                    pos.side,
                                    pos.entry_price,
                                    current_price,
                                    profit_percent
                                );

                                // 检查亏损情况，必要时发送提醒或执行止损
                                let position_key = format!("{}:{}", pos.symbol, pos.side);
                                current_position_keys.insert(position_key.clone());

                                if profit_rate <= alert_percent {
                                    if alerted_positions.insert(position_key.clone()) {
                                        let message = format!(
                                            "🚨 <b>止损预警</b>\n\n\
                                             💰 交易对: <code>{}</code>\n\
                                             📊 方向: <code>{}</code>\n\
                                             📉 浮亏: <code>{:.1}%</code>\n\
                                             💵 入场价: <code>{:.4} USDT</code>\n\
                                             💵 当前价: <code>{:.4} USDT</code>\n\
                                             🕐 时间: {}\n\n\
                                             ⚠️ 未执行自动平仓，请尽快手动处理。",
                                            pos.symbol,
                                            pos.side,
                                            profit_percent,
                                            pos.entry_price,
                                            current_price,
                                            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                                        );

                                        warn!(
                                            "⚠️ {} 亏损达到{:.1}%，发送Telegram提醒",
                                            pos.symbol, profit_percent
                                        );

                                        if let Err(err) = send_telegram_alert(
                                            telegram_client,
                                            telegram_config,
                                            &message,
                                        )
                                        .await
                                        {
                                            error!("❌ Telegram提醒发送失败: {}", err);
                                            alerted_positions.remove(&position_key);
                                        } else {
                                            info!("✅ 已发送Telegram亏损提醒: {}", pos.symbol);
                                        }
                                    }
                                } else {
                                    alerted_positions.remove(&position_key);
                                }

                                if auto_close_enabled && profit_rate <= stop_loss_percent {
                                    warn!(
                                        "⚠️ {} 亏损达到{:.1}%，触发自动止损保护！",
                                        pos.symbol, profit_percent
                                    );

                                    if lock_manager.try_acquire_lock(
                                        &pos.symbol,
                                        "close",
                                        "profit_monitor",
                                        60,
                                    )? {
                                        match client
                                            .close_position(&pos.symbol, &pos.side, pos.size)
                                            .await
                                        {
                                            Ok(_) => {
                                                info!(
                                                    "✅ 止损保护成功: {} {} {:.4} (亏损: {:.2}%)",
                                                    pos.symbol, pos.side, pos.size, profit_percent
                                                );
                                            }
                                            Err(e) => {
                                                error!("❌ 止损执行失败: {} - {}", pos.symbol, e);
                                            }
                                        }
                                        lock_manager
                                            .release_lock(&pos.symbol, "close", "profit_monitor")
                                            .ok();
                                    } else {
                                        warn!(
                                            "⚠️  {} 平仓操作被锁定，可能signal_trader正在处理",
                                            pos.symbol
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                error!("❌ 获取{}价格失败: {}", pos.symbol, e);
                            }
                        }
                    }

                    alerted_positions.retain(|key| current_position_keys.contains(key));
                }
            }
            Err(e) => {
                error!("❌ 获取持仓失败: {}", e);
            }
        }

        // 等待30秒后再次检查
        sleep(Duration::from_secs(30)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("🛡️ 启动止损监控系统");
    println!("════════════════════════════════════════");

    // 读取配置
    let binance_api_key = env::var("BINANCE_API_KEY")?;
    let binance_secret = env::var("BINANCE_SECRET_KEY")?;
    let testnet = env::var("BINANCE_TESTNET")?.parse::<bool>()?;
    let stop_loss_percent = env::var("SIGNAL_STOP_LOSS_PERCENT")?.parse::<f64>()?;
    let leverage: u32 = env::var("SIGNAL_LEVERAGE")?.parse()?;
    let alert_percent = env::var("STOP_LOSS_ALERT_PERCENT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(-0.5);
    let auto_close_enabled = env::var("SIGNAL_AUTO_STOP_LOSS_ENABLED")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let telegram_config = TelegramConfig {
        bot_token: env::var("TELEGRAM_BOT_TOKEN")?,
        chat_id: env::var("TELEGRAM_CHAT_ID")?,
    };
    let telegram_client = Client::new();

    println!(
        "🔑 Binance 环境: {}",
        if testnet { "测试网" } else { "主网" }
    );
    println!("📉 亏损提醒阈值: {:.0}%", alert_percent * 100.0);
    println!("⚠️  自动止损阈值: {:.0}%", stop_loss_percent * 100.0);
    println!(
        "🤖 自动平仓: {}",
        if auto_close_enabled {
            "启用"
        } else {
            "禁用"
        }
    );
    println!("⚡ 杠杆: {}x", leverage);
    println!("⏰ 监控频率: 每30秒");
    println!("════════════════════════════════════════\n");

    // 初始化Binance客户端
    let client = BinanceClient::new(binance_api_key, binance_secret, testnet);

    // 初始化健康监控
    let health_monitor = HealthMonitor::new();
    health_monitor
        .update_status("profit_monitor", "starting")
        .ok();

    // 初始化交易锁管理器
    let lock_manager = TradingLockManager::new();
    lock_manager.cleanup_expired_locks().ok();

    // 验证连接
    match client.get_account_info().await {
        Ok(_) => {
            println!("✅ Binance 连接成功");
        }
        Err(e) => {
            error!("❌ Binance 连接失败: {}", e);
            return Err(e);
        }
    }

    println!("🔍 开始监控持仓亏损保护...\n");

    // 开始监控
    monitor_positions(
        &client,
        stop_loss_percent,
        alert_percent,
        leverage,
        &health_monitor,
        &lock_manager,
        &telegram_client,
        &telegram_config,
        auto_close_enabled,
    )
    .await?;

    Ok(())
}
