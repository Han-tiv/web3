# Gemini ETH 分析器仓位约束
- 日期：2025-11-13
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 现在固定使用 2 USDT 本金（`CAPITAL = 2.0`）。
- 在 `execute_trade_action` 中必须先调用 `binance.set_margin_type(SYMBOL, "ISOLATED")` 再 `set_leverage`，保证逐仓模式与风险隔离。
- 触发条件：运行 Gemini ETH 分析器进行限价建仓时。