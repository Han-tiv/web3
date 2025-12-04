//! AI Prompt 构建公共模块
//!
//! 提取 DeepSeek 和 Gemini 中的公共 prompt 构建逻辑，
//! 减少代码重复，提高可维护性。

use crate::ai_core::deepseek::{Kline, TechnicalIndicators};

/// Prompt 构建器 - 提供公共的 prompt 构建功能
pub struct PromptBuilder;

impl PromptBuilder {
    /// 格式化 K线数据为文本
    ///
    /// # 参数
    /// - `klines`: K线数据数组
    /// - `label`: 时间周期标签（如 "5m", "15m", "1h"）
    /// - `limit`: 显示的K线数量
    pub fn format_klines(klines: &[Kline], label: &str, limit: usize) -> String {
        let recent: Vec<&Kline> = klines.iter().rev().take(limit).collect();
        let mut lines = vec![format!("\n📊 {}周期 K线 (最近{}根):", label, recent.len())];

        for (i, kline) in recent.iter().rev().enumerate() {
            let timestamp = kline.timestamp / 1000;
            let time_str = chrono::DateTime::from_timestamp(timestamp, 0)
                .map(|dt| dt.naive_utc())
                .map(|dt| dt.format("%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let change_pct = if kline.open > 0.0 {
                ((kline.close - kline.open) / kline.open) * 100.0
            } else {
                0.0
            };

            let candle_type = if kline.close > kline.open {
                "🟢"
            } else if kline.close < kline.open {
                "🔴"
            } else {
                "⚪"
            };

            lines.push(format!(
                "  {} {}: O={:.4} H={:.4} L={:.4} C={:.4} ({:+.2}%) Vol={:.0}",
                candle_type,
                time_str,
                kline.open,
                kline.high,
                kline.low,
                kline.close,
                change_pct,
                kline.volume
            ));

            if i >= limit - 1 {
                break;
            }
        }

        lines.join("\n")
    }

    /// 格式化技术指标为文本
    pub fn format_indicators(indicators: &TechnicalIndicators) -> String {
        format!(
            r#"
📊 技术指标:
- RSI(14): {:.2} {}
- MACD: {:.4} (信号线: {:.4}) {}
- SMA(5/20/50): {:.4} / {:.4} / {:.4}
- 布林带: 上轨={:.4}, 中轨={:.4}, 下轨={:.4}"#,
            indicators.rsi,
            Self::interpret_rsi(indicators.rsi),
            indicators.macd,
            indicators.macd_signal,
            Self::interpret_macd_simple(indicators),
            indicators.sma_5,
            indicators.sma_20,
            indicators.sma_50,
            indicators.bb_upper,
            indicators.bb_middle,
            indicators.bb_lower
        )
    }

    /// 解释 RSI 值
    fn interpret_rsi(rsi: f64) -> &'static str {
        if rsi > 70.0 {
            "(超买)"
        } else if rsi < 30.0 {
            "(超卖)"
        } else if rsi > 60.0 {
            "(偏强)"
        } else if rsi < 40.0 {
            "(偏弱)"
        } else {
            "(中性)"
        }
    }

    /// 解释 MACD（简化版，不依赖histogram）
    fn interpret_macd_simple(indicators: &TechnicalIndicators) -> &'static str {
        if indicators.macd > indicators.macd_signal {
            "(多头)"
        } else if indicators.macd < indicators.macd_signal {
            "(空头)"
        } else {
            "(中性)"
        }
    }

    /// 识别关键位（支撑和阻力）
    ///
    /// 基于 K线的影线聚集识别关键价格位
    pub fn identify_key_levels(klines: &[Kline], current_price: f64) -> String {
        if klines.is_empty() {
            return String::from("K线数据不足");
        }

        // 收集所有高低点
        let mut highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
        let mut lows: Vec<f64> = klines.iter().map(|k| k.low).collect();

        highs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lows.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // 找到最近的阻力位（大于当前价的最小高点）
        let resistance = highs
            .iter()
            .find(|&&h| h > current_price * 1.001)
            .copied()
            .unwrap_or(current_price * 1.05);

        // 找到最近的支撑位（小于当前价的最大低点）
        let support = lows
            .iter()
            .rev()
            .find(|&&l| l < current_price * 0.999)
            .copied()
            .unwrap_or(current_price * 0.95);

        let resistance_dist = ((resistance - current_price) / current_price) * 100.0;
        let support_dist = ((current_price - support) / current_price) * 100.0;

        format!(
            r#"
🎯 关键位分析:
- 上方阻力: ${:.4} (距离+{:.2}%)
- 下方支撑: ${:.4} (距离-{:.2}%)
- 当前价格: ${:.4}"#,
            resistance, resistance_dist, support, support_dist, current_price
        )
    }

    /// 构建资金流向说明
    pub fn build_fund_flow_text(alert_type: &str, fund_type: &str, alert_message: &str) -> String {
        format!(
            r#"
💰 资金异动信号:
- 信号类型: {} (资金流入=买入机会, 资金出逃=卖出信号)
- 资金类型: {}
- 原始消息: {}"#,
            alert_type, fund_type, alert_message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_rsi() {
        assert_eq!(PromptBuilder::interpret_rsi(75.0), "(超买)");
        assert_eq!(PromptBuilder::interpret_rsi(25.0), "(超卖)");
        assert_eq!(PromptBuilder::interpret_rsi(50.0), "(中性)");
    }

    #[test]
    fn test_format_klines_empty() {
        let klines: Vec<Kline> = vec![];
        let result = PromptBuilder::format_klines(&klines, "5m", 10);
        assert!(result.contains("5m周期"));
    }
}
