## 2025-02-19
- 更新 `apps/rust-trading-bot/src/binance_client.rs` 中 `get_symbol_trading_rules`，同时解析 `LOT_SIZE` 与 `PRICE_FILTER`，返回 `tick_size` 并缓存。
- `limit_order` 在格式化之前按 `tick_size` 对齐价格，日志输出也记录对齐后的价格，避免 Binance 因精度不符产生 -4014 错误。