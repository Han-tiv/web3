use crate::ai_core::deepseek::{Kline, TechnicalIndicators};
use crate::ai_core::prompt_builder::PromptBuilder;
use serde_json::Value;

// 备用 prompt 构建函数,供 V2 批量评估实验使用
#[allow(dead_code)]
pub fn build_batch_evaluation_prompt(
    positions: &[(
        String,
        String,
        f64,
        f64,
        f64,
        f64,
        Vec<Kline>,
        Vec<Kline>,
        Vec<Kline>,
        TechnicalIndicators,
    )],
) -> String {
    let summarize_klines = |klines: &[Kline], limit: usize| -> Vec<Value> {
        let mut recent: Vec<&Kline> = klines.iter().rev().take(limit).collect();
        recent.reverse();
        recent
            .into_iter()
            .map(|kline| {
                serde_json::json!({
                    "timestamp": kline.timestamp,
                    "open": kline.open,
                    "high": kline.high,
                    "low": kline.low,
                    "close": kline.close,
                    "volume": kline.volume,
                    "quote_volume": kline.quote_volume,
                    "taker_buy_volume": kline.taker_buy_volume,
                    "taker_buy_quote_volume": kline.taker_buy_quote_volume,
                })
            })
            .collect()
    };

    let mut payload: Vec<Value> = Vec::with_capacity(positions.len());

    for (
        symbol,
        side,
        entry_price,
        current_price,
        profit_pct,
        duration,
        klines_5m,
        klines_15m,
        klines_1h,
        indicators,
    ) in positions
    {
        let key_levels = PromptBuilder::identify_key_levels(klines_1h, *current_price);

        payload.push(serde_json::json!({
            "symbol": symbol,
            "side": side,
            "entry_price": entry_price,
            "current_price": current_price,
            "profit_pct": profit_pct,
            "duration_hours": duration,
            "market_data": {
                "klines_5m": summarize_klines(klines_5m, 5),
                "klines_15m": summarize_klines(klines_15m, 5),
                "klines_1h": summarize_klines(klines_1h, 5),
                "indicators": {
                    "sma_5": indicators.sma_5,
                    "sma_20": indicators.sma_20,
                    "sma_50": indicators.sma_50,
                    "rsi": indicators.rsi,
                    "macd": indicators.macd,
                    "macd_signal": indicators.macd_signal,
                    "bb_upper": indicators.bb_upper,
                    "bb_middle": indicators.bb_middle,
                    "bb_lower": indicators.bb_lower
                },
                "key_levels_analysis": key_levels
            }
        }));
    }

    let json_payload = serde_json::to_string_pretty(&payload).unwrap_or_default();

    format!(
        r#"你是专业加密货币交易员，请批量评估以下持仓。

数据格式(JSON数组):
{}

请对每个持仓返回以下JSON格式的决策(返回JSON数组):
[
    {{
        "symbol": "BTCUSDT",
        "action": "HOLD|CLOSE|PARTIAL_CLOSE|ADD",
        "confidence": "HIGH|MEDIUM|LOW",
        "quantity_pct": 0-100,
        "reason": "简短理由",
        "new_stop_loss": 0.0,
        "new_take_profit": 0.0
    }},
    ...
]

决策逻辑:
1. 关键位止盈: 遇到强阻力/支撑减仓
2. 趋势跟随: 趋势良好继续持有
3. 风险控制: 跌破关键位止损
"#,
        json_payload
    )
}
