# Valuescan V2 切换逻辑
- `src/bin/integrated_ai_trader.rs` 通过 `USE_VALUESCAN_V2` 环境变量选择 Gemini 开仓分析版本，默认关闭。
- 开启时调用 `build_entry_analysis_prompt_v2` + `analyze_market_v2`，得到 `TradingSignalV2` 后再转换为 `TradingSignal` 并输出 Valuescan 评分日志，兼容现有下单流程。