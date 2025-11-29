//! 工具函数模块
//!
//! 包含通用的辅助函数和常量定义

use chrono::{DateTime, Utc};

/// MEME币种列表 - 触发更严格风控
pub const MEME_COINS: [&str; 7] = [
    "PUMPUSDT",
    "GIGGLEUSDT",
    "POPCATUSDT",
    "WIFUSDT",
    "SHIBUSDT",
    "DOGEUSDT",
    "PEPEUSDT",
];

/// 判断是否为MEME币
///
/// # Arguments
/// * `symbol` - 交易对符号，如 "BTCUSDT"
///
/// # Returns
/// * `true` - 是MEME币，需要更严格风控
/// * `false` - 普通币种
pub fn is_meme_coin(symbol: &str) -> bool {
    MEME_COINS
        .iter()
        .any(|meme| meme.eq_ignore_ascii_case(symbol))
}

/// 将毫秒时间戳转换为UTC时间
///
/// # Arguments
/// * `ms` - 毫秒时间戳
///
/// # Returns
/// * `DateTime<Utc>` - UTC时间，如果转换失败则返回当前时间
pub fn timestamp_ms_to_datetime(ms: i64) -> DateTime<Utc> {
    let secs = ms.div_euclid(1000);
    let nsecs = (ms.rem_euclid(1000) as u32) * 1_000_000;
    DateTime::from_timestamp(secs, nsecs).unwrap_or_else(|| Utc::now())
}

/// 归一化信号类型
///
/// # Arguments
/// * `raw` - 原始信号类型字符串
///
/// # Returns
/// * 归一化后的信号类型：alpha, fomo, inflow, unknown
pub fn normalize_signal_type(raw: &str) -> &'static str {
    match raw.to_lowercase().as_str() {
        "alpha" | "alpha_fomo" => "alpha",
        "fomo" => "fomo",
        "fund_inflow" => "inflow",
        _ => "unknown",
    }
}

/// 将置信度映射到评分
///
/// # Arguments
/// * `confidence` - 置信度等级：HIGH, MEDIUM, LOW
///
/// # Returns
/// * 数值评分：HIGH=8.0, MEDIUM=6.5, LOW=5.0, 默认=6.0
pub fn map_confidence_to_score(confidence: &str) -> f64 {
    match confidence.to_uppercase().as_str() {
        "HIGH" => 8.0,
        "MEDIUM" => 6.5,
        "LOW" => 5.0,
        _ => 6.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_meme_coin() {
        assert!(is_meme_coin("PUMPUSDT"));
        assert!(is_meme_coin("pumpusdt")); // 不区分大小写
        assert!(is_meme_coin("DOGEUSDT"));
        assert!(!is_meme_coin("BTCUSDT"));
        assert!(!is_meme_coin("ETHUSDT"));
    }

    #[test]
    fn test_normalize_signal_type() {
        assert_eq!(normalize_signal_type("alpha"), "alpha");
        assert_eq!(normalize_signal_type("ALPHA"), "alpha");
        assert_eq!(normalize_signal_type("alpha_fomo"), "alpha");
        assert_eq!(normalize_signal_type("fomo"), "fomo");
        assert_eq!(normalize_signal_type("FOMO"), "fomo");
        assert_eq!(normalize_signal_type("fund_inflow"), "inflow");
        assert_eq!(normalize_signal_type("unknown_type"), "unknown");
    }

    #[test]
    fn test_map_confidence_to_score() {
        assert_eq!(map_confidence_to_score("HIGH"), 8.0);
        assert_eq!(map_confidence_to_score("high"), 8.0);
        assert_eq!(map_confidence_to_score("MEDIUM"), 6.5);
        assert_eq!(map_confidence_to_score("LOW"), 5.0);
        assert_eq!(map_confidence_to_score("UNKNOWN"), 6.0);
    }

    #[test]
    fn test_timestamp_ms_to_datetime() {
        let ms = 1638360000000_i64; // 2021-12-01 10:00:00 UTC
        let dt = timestamp_ms_to_datetime(ms);
        assert_eq!(dt.timestamp(), 1638360000);
    }
}
