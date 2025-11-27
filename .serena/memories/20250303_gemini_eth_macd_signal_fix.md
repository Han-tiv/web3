# 任务
修复 `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 中 MACD 信号线计算。

# 关键信息
- `calculate_macd` 现在在价格数据少于 26 根时返回 0 值，避免使用不充分的数据。
- MACD 计算会构建完整的 MACD 历史序列，并对其执行 9 日 EMA 来获得 signal line，最终得到更准确的 histogram。
- 依赖现有 `calculate_ema` 函数，未新增外部依赖或配置。