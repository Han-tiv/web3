## 2025-11-06
- 更新 `apps/rust-trading-bot/src/okx_client.rs` 中构造 `TradingRules` 的默认值，新增 `tick_size` 字段并默认设为 `0.0001`，与其他交易所实现保持一致，方便后续统一按价格步长对齐限价单价格。