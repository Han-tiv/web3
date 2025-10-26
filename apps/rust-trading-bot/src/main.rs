mod binance_client;
mod copy_trader;
mod telegram_bot;

use anyhow::Result;
use dotenv::dotenv;
use log::info;
use std::env;

use crate::binance_client::BinanceClient;
use crate::copy_trader::{CopyTradeConfig, CopyTrader};
use crate::telegram_bot::TelegramBot;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv().ok();

    // 初始化日志
    env_logger::init();

    info!("🚀 Rust Trading Bot 启动中...");

    // 读取配置
    let config = load_config()?;
    let multi_asset_mode = env::var("SIGNAL_MULTI_ASSET_MODE")
        .or_else(|_| env::var("MULTI_ASSET_MODE"))
        .unwrap_or_else(|_| "SINGLE".to_string());

    info!(
        "⚙️ 交易参数: 使用 {:.2} USDT 固定保证金，{}x 杠杆，{}，{}仓位模式 ({} 保证金资产)",
        config.fixed_margin_usdt,
        config.leverage,
        if config.margin_type.to_uppercase() == "ISOLATED" {
            "逐仓"
        } else {
            "全仓"
        },
        if config.dual_side_position {
            "双向"
        } else {
            "单向"
        },
        match multi_asset_mode.to_uppercase().as_str() {
            "MULTI" | "TRUE" | "1" => "多币种",
            _ => "单币种",
        }
    );

    // 创建Binance客户端
    let leader_client = BinanceClient::new(
        env::var("LEADER_API_KEY")?,
        env::var("LEADER_SECRET_KEY")?,
        config.testnet,
    );

    let follower_client = BinanceClient::new(
        env::var("BINANCE_API_KEY")?,
        env::var("BINANCE_SECRET_KEY")?,
        config.testnet,
    );

    info!("✅ Binance客户端初始化完成");

    // 创建跟单交易器
    let copy_config = CopyTradeConfig {
        copy_ratio: config.copy_ratio,
        max_position_size: config.max_position_size,
        leverage: config.leverage,
        enable_stop_loss: true,
        stop_loss_percent: 0.05, // 5% 止损
        fixed_margin_usdt: config.fixed_margin_usdt,
        margin_type: config.margin_type.clone(),
        dual_side_position: config.dual_side_position,
    };

    let copy_trader = CopyTrader::new(leader_client, follower_client, copy_config);

    info!("✅ 跟单系统初始化完成");

    // 创建Telegram Bot
    let telegram_token = env::var("TELOXIDE_TOKEN")?;
    let telegram_bot = TelegramBot::new(telegram_token, copy_trader);

    info!("✅ Telegram Bot初始化完成");

    // 运行Telegram Bot
    telegram_bot.run().await;

    Ok(())
}

struct Config {
    testnet: bool,
    copy_ratio: f64,
    max_position_size: f64,
    leverage: u32,
    fixed_margin_usdt: f64,
    margin_type: String,
    dual_side_position: bool,
}

fn load_config() -> Result<Config> {
    let leverage = env::var("SIGNAL_LEVERAGE")
        .or_else(|_| env::var("LEVERAGE"))
        .unwrap_or_else(|_| "3".to_string())
        .parse()?;

    let fixed_margin_usdt = env::var("SIGNAL_MARGIN")
        .or_else(|_| env::var("COPY_MARGIN_USDT"))
        .unwrap_or_else(|_| "2".to_string())
        .parse()?;

    let margin_type = env::var("SIGNAL_MARGIN_TYPE")
        .or_else(|_| env::var("TRADING_MARGIN_TYPE"))
        .unwrap_or_else(|_| "ISOLATED".to_string());

    let position_mode_raw = env::var("SIGNAL_POSITION_MODE")
        .or_else(|_| env::var("POSITION_MODE"))
        .unwrap_or_else(|_| "SINGLE".to_string());
    let dual_side_position = matches!(position_mode_raw.trim().to_uppercase().as_str(), "DUAL");

    Ok(Config {
        testnet: env::var("BINANCE_TESTNET")
            .unwrap_or_else(|_| "true".to_string())
            .parse()?,
        copy_ratio: env::var("COPY_RATIO")
            .unwrap_or_else(|_| "0.5".to_string())
            .parse()?,
        max_position_size: env::var("MAX_POSITION_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()?,
        leverage,
        fixed_margin_usdt,
        margin_type,
        dual_side_position,
    })
}
