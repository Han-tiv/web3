## 2025-02-19
- 调整 `apps/rust-trading-bot/src/binance_client.rs` 中 `open_long`、`open_short` 的限价计算，调用 `get_symbol_trading_rules` 并按 `price_precision` 格式化价格，确保符合 Binance tick size 要求，避免 -4014 错误。
- 日志输出使用格式化后的字符串，便于排查精度问题。