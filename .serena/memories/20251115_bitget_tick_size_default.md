## 2025-11-15
- 更新 `apps/rust-trading-bot/src/bitget_client.rs` 的 `TradingRules` 构造，新增 `tick_size` 字段并默认设置为 0.0001，以保持与其他交易所实现一致的价格对齐策略。