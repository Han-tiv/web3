//! 数据校验工具函数
//!
//! 目前包含入场区间校验以及 MEME 币种识别，提供独立的风控判断能力。

use anyhow::Result;
use log::{info, warn};

use super::super::{TechnicalIndicators, MEME_COINS};

/// 【P0-2】校验当前价格是否仍处于允许的入场区间，避免信号延迟导致追高。
///
/// 该函数遵循 `trader.rs` 中相同的风控逻辑，包含：
/// 1. 信号价偏离检查（>2% 则拒绝）
/// 2. 根据 AI 覆盖状态动态扩展容差
/// 3. RSI 超买过滤
#[allow(dead_code)]
pub async fn validate_entry_zone(
    signal_price: f64,
    current_price: f64,
    entry_zone: (f64, f64),
    indicators: &TechnicalIndicators,
    is_ai_override: bool,
) -> Result<bool> {
    if signal_price > 0.0 {
        let deviation = (current_price - signal_price).abs() / signal_price;
        if deviation > 0.02 {
            warn!("❌ 信号延迟过大: 偏离{:.2}%, 拒绝入场", deviation * 100.0);
            return Ok(false);
        }
    } else {
        warn!(
            "⚠️ signal_price为0,跳过偏离度检查 (当前价: ${:.4})",
            current_price
        );
    }

    let (entry_zone_min, entry_zone_max) = entry_zone;
    let price_tolerance = if is_ai_override {
        let rsi = indicators.rsi;
        let price_range = (entry_zone_max - entry_zone_min) / entry_zone_min * 100.0;

        if rsi > 65.0 || price_range > 5.0 {
            0.25
        } else if rsi > 45.0 {
            0.20
        } else {
            0.15
        }
    } else {
        0.03
    };

    let extended_min = entry_zone_min * (1.0 - price_tolerance);
    let extended_max = entry_zone_max * (1.0 + price_tolerance);

    if current_price < extended_min || current_price > extended_max {
        warn!(
            "❌ 价格不在入场区 [{:.4}, {:.4}] (扩展), 当前{:.4}, 拒绝入场",
            extended_min, extended_max, current_price
        );
        return Ok(false);
    }

    if is_ai_override && (current_price < entry_zone_min || current_price > entry_zone_max) {
        info!(
            "⚠️  价格超出标准入场区,但在AI动态容差范围内 ({:.1}%, RSI={:.1})",
            price_tolerance * 100.0,
            indicators.rsi
        );
        info!(
            "   标准区间: [{:.4}, {:.4}]",
            entry_zone_min, entry_zone_max
        );
        info!("   扩展区间: [{:.4}, {:.4}]", extended_min, extended_max);
        info!("   当前价格: {:.4}", current_price);
    }

    if indicators.rsi > 75.0 {
        warn!("❌ RSI严重超买 {:.1}, 拒绝入场", indicators.rsi);
        return Ok(false);
    }

    Ok(true)
}

/// 判断给定交易对是否属于 MEME 币种（不区分大小写）。
pub fn is_meme_coin(symbol: &str) -> bool {
    MEME_COINS
        .iter()
        .any(|meme| meme.eq_ignore_ascii_case(symbol))
}
