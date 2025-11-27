# Gemini ETH 分析器动态本金与盈利记录
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 移除了 DRY-RUN 模式，实盘执行前会调用 `calculate_dynamic_capital`，依据 `trade_profit_history` 最近一次盈亏在 2-5 USDT 间动态调整本金。
- 关键信息存储于数据库新表 `trade_profit_history`，通过 `Database::record_trade_profit` 和 `Database::get_last_profit` 读写，该表在 `bootstrap` 中自动初始化。
- 平仓后会调用 `record_trade_profit` 写入盈亏，后续可依此继续优化资金管理及 Prompt 风控描述。