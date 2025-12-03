//! 集成AI交易系统配置常量
//!
//! 从 trader.rs 提取的所有配置常量，便于统一管理和调整

use lazy_static::lazy_static;
use std::env;

/// 持仓检查间隔（秒）- P1优化: 从600s减少到180s,提升风控响应速度
pub const POSITION_CHECK_INTERVAL_SECS: u64 = 180;

/// 是否使用增强版持仓分析逻辑（保留供后续切换）
#[allow(dead_code)]
pub const USE_ENHANCED_ANALYSIS: bool = false;

/// 波动率缓存TTL（秒）
#[allow(dead_code)]
pub const VOLATILITY_30DAY_LOOKBACK_DAYS: i64 = 30; // 波动率计算周期
pub const VOLATILITY_CACHE_TTL_SECS: u64 = 3600; // 波动率缓存1小时

/// 波动率计算超时（秒）
#[allow(dead_code)]
pub const VOLATILITY_TIMEOUT_SECS: u64 = 5;

/// 交易配置波动率计算回溯周期
#[allow(dead_code)]
pub const VOLATILITY_LOOKBACK: usize = 20;

/// 默认波动率百分比
#[allow(dead_code)]
pub const DEFAULT_VOLATILITY_PERCENT: f64 = 5.0; // 默认波动率百分比

/// MEME 币种列表（触发更严格风控）
#[allow(dead_code)]
pub const MEME_COINS: [&str; 7] = [
    "PUMPUSDT",
    "GIGGLEUSDT",
    "POPCATUSDT",
    "WIFUSDT",
    "SHIBUSDT",
    "DOGEUSDT",
    "PEPEUSDT",
];

// 是否使用 Valuescan V2 方法论
lazy_static! {
    pub static ref USE_VALUESCAN_V2: bool = env::var("USE_VALUESCAN_V2")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);
}

/// 交易配置
pub struct TradingConfig {
    /// 最小仓位（USDT）
    pub min_position_usdt: f64,
    /// 最大仓位（USDT）
    pub max_position_usdt: f64,
    /// 最小杠杆
    pub min_leverage: u32,
    /// 最大杠杆
    pub max_leverage: u32,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            min_position_usdt: 5.0, // 单笔固定 5 USDT (满足Binance最小订单要求)
            max_position_usdt: 5.0,
            min_leverage: 5,  // Low信心=5x
            max_leverage: 15, // High信心=15x, Medium信心=10x
        }
    }
}

/// 内存管理配置
pub struct MemoryConfig {
    /// 最多追踪币种数量
    pub max_tracked_coins: usize,
    /// 币种追踪过期时间（小时）
    pub coin_ttl_hours: i64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_tracked_coins: 100, // 最多追踪 100 个币种
            coin_ttl_hours: 24,     // 24 小时后自动过期
        }
    }
}

/// Alpha/FOMO 关键词配置
pub struct KeywordConfig {
    pub alpha_keywords: Vec<String>,
    pub fomo_keywords: Vec<String>,
}

impl Default for KeywordConfig {
    fn default() -> Self {
        Self {
            alpha_keywords: vec![
                "alpha".to_string(),
                "新币".to_string(),
                "上线".to_string(),
                "首发".to_string(),
                "binance".to_string(),
                "币安".to_string(),
            ],
            fomo_keywords: vec![
                "暴涨".to_string(),
                "拉升".to_string(),
                "突破".to_string(),
                "异动".to_string(),
                "急拉".to_string(),
                "爆发".to_string(),
            ],
        }
    }
}
