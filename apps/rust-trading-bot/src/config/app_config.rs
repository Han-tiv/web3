//! Application Configuration
//!
//! 中心化的配置管理系统，支持从环境变量和配置文件加载

use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;

/// 应用总配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// 交易配置
    pub trading: TradingConfig,
    /// 风控配置
    pub risk: RiskConfig,
    /// AI配置
    pub ai: AIConfig,
    /// 交易所配置
    pub exchange: ExchangeConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
}

/// 交易配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    /// 最小仓位（USDT）
    #[serde(default = "default_min_position")]
    pub min_position_usdt: f64,
    /// 最大仓位（USDT）
    #[serde(default = "default_max_position")]
    pub max_position_usdt: f64,
    /// 最小杠杆
    #[serde(default = "default_min_leverage")]
    pub min_leverage: u32,
    /// 最大杠杆
    #[serde(default = "default_max_leverage")]
    pub max_leverage: u32,
    /// 初始余额
    #[serde(default = "default_initial_balance")]
    pub initial_balance: f64,
}

/// 风控配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    /// 单日最大亏损（USDT）
    #[serde(default = "default_max_daily_loss")]
    pub max_daily_loss: f64,
    /// 最大持仓数
    #[serde(default = "default_max_positions")]
    pub max_positions: usize,
    /// 最大追踪币种数
    #[serde(default = "default_max_tracked_coins")]
    pub max_tracked_coins: usize,
    /// 币种追踪过期时间（小时）
    #[serde(default = "default_coin_ttl_hours")]
    pub coin_ttl_hours: i64,
}

/// AI配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIConfig {
    /// DeepSeek API Key
    pub deepseek_api_key: String,
    /// Gemini API Key
    pub gemini_api_key: String,
    /// Grok API Key (可选)
    pub grok_api_key: Option<String>,
}

/// 交易所配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeConfig {
    /// Binance API Key
    pub binance_api_key: String,
    /// Binance Secret
    pub binance_secret: String,
    /// 是否使用测试网
    #[serde(default)]
    pub testnet: bool,
}

/// 监控配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// 持仓检查间隔（秒）
    #[serde(default = "default_position_check_interval")]
    pub position_check_interval_secs: u64,
    /// 清理间隔（分钟）
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_mins: u64,
    /// Web服务器端口
    #[serde(default = "default_web_port")]
    pub web_port: u16,
    /// Telegram Bot Token (可选)
    pub telegram_bot_token: Option<String>,
}

// 默认值函数
fn default_min_position() -> f64 { 5.0 }
fn default_max_position() -> f64 { 5.0 }
fn default_min_leverage() -> u32 { 5 }
fn default_max_leverage() -> u32 { 15 }
fn default_initial_balance() -> f64 { 50.03 }
fn default_max_daily_loss() -> f64 { 100.0 }
fn default_max_positions() -> usize { 5 }
fn default_max_tracked_coins() -> usize { 100 }
fn default_coin_ttl_hours() -> i64 { 24 }
fn default_position_check_interval() -> u64 { 180 }
fn default_cleanup_interval() -> u64 { 60 }
fn default_web_port() -> u16 { 8081 }

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            trading: TradingConfig {
                min_position_usdt: env::var("TRADING_MIN_POSITION_USDT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_min_position),
                max_position_usdt: env::var("TRADING_MAX_POSITION_USDT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_position),
                min_leverage: env::var("TRADING_MIN_LEVERAGE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_min_leverage),
                max_leverage: env::var("TRADING_MAX_LEVERAGE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_leverage),
                initial_balance: env::var("TRADING_INITIAL_BALANCE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_initial_balance),
            },
            risk: RiskConfig {
                max_daily_loss: env::var("RISK_MAX_DAILY_LOSS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_daily_loss),
                max_positions: env::var("RISK_MAX_POSITIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_positions),
                max_tracked_coins: env::var("RISK_MAX_TRACKED_COINS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_tracked_coins),
                coin_ttl_hours: env::var("RISK_COIN_TTL_HOURS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_coin_ttl_hours),
            },
            ai: AIConfig {
                deepseek_api_key: env::var("DEEPSEEK_API_KEY")?,
                gemini_api_key: env::var("GEMINI_API_KEY_1")?,
                grok_api_key: env::var("GROK_API_KEY").ok(),
            },
            exchange: ExchangeConfig {
                binance_api_key: env::var("BINANCE_API_KEY")?,
                binance_secret: env::var("BINANCE_SECRET")?,
                testnet: env::var("BINANCE_TESTNET")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
            monitoring: MonitoringConfig {
                position_check_interval_secs: env::var("MONITORING_POSITION_CHECK_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_position_check_interval),
                cleanup_interval_mins: env::var("MONITORING_CLEANUP_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_cleanup_interval),
                web_port: env::var("MONITORING_WEB_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_web_port),
                telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN").ok(),
            },
        })
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            trading: TradingConfig {
                min_position_usdt: default_min_position(),
                max_position_usdt: default_max_position(),
                min_leverage: default_min_leverage(),
                max_leverage: default_max_leverage(),
                initial_balance: default_initial_balance(),
            },
            risk: RiskConfig {
                max_daily_loss: default_max_daily_loss(),
                max_positions: default_max_positions(),
                max_tracked_coins: default_max_tracked_coins(),
                coin_ttl_hours: default_coin_ttl_hours(),
            },
            ai: AIConfig {
                deepseek_api_key: String::new(),
                gemini_api_key: String::new(),
                grok_api_key: None,
            },
            exchange: ExchangeConfig {
                binance_api_key: String::new(),
                binance_secret: String::new(),
                testnet: false,
            },
            monitoring: MonitoringConfig {
                position_check_interval_secs: default_position_check_interval(),
                cleanup_interval_mins: default_cleanup_interval(),
                web_port: default_web_port(),
                telegram_bot_token: None,
            },
        }
    }
}
