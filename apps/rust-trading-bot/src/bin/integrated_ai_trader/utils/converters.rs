//! 数据转换工具函数
//!
//! 提供毫秒时间戳转换、AI信号标准化、置信度评分映射等能力。

use chrono::{DateTime, Utc};

/// 将毫秒时间戳安全转换为 UTC 时间，若解析失败则回退到当前时间。
pub fn timestamp_ms_to_datetime(ms: i64) -> DateTime<Utc> {
    let secs = ms.div_euclid(1000);
    let nsecs = (ms.rem_euclid(1000) as u32) * 1_000_000;
    DateTime::<Utc>::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
}

/// 将 AI 输出的原始动作统一映射为 BUY/SELL/HOLD/CLOSE，确保前端展示一致性。
pub fn normalize_signal_type(raw: &str) -> &'static str {
    let normalized = raw.trim().to_ascii_uppercase();

    match normalized.as_str() {
        "BUY" => "BUY",
        "SELL" => "SELL",
        "HOLD" => "HOLD",
        "CLOSE" => "CLOSE",
        "FULL_CLOSE" | "PARTIAL_CLOSE" => "CLOSE",
        "SET_LIMIT_ORDER" | "SKIP" | "WAIT" | "WAIT_FOR_SIGNAL" => "HOLD",
        value if value.contains("BUY") => "BUY",
        value if value.contains("SELL") => "SELL",
        value if value.contains("CLOSE") => "CLOSE",
        _ => "HOLD",
    }
}

/// 将 AI 置信度字符串映射为 0.0-1.0 的分值，统一前端展示口径。
pub fn map_confidence_to_score(confidence: &str) -> f64 {
    let trimmed = confidence.trim();
    let normalized = trimmed.to_ascii_uppercase();

    match normalized.as_str() {
        "HIGH" => 0.9,
        "MEDIUM" => 0.7,
        "LOW" => 0.5,
        _ => trimmed
            .parse::<f64>()
            .map(|value| value.clamp(0.0, 1.0))
            .unwrap_or(0.0),
    }
}
