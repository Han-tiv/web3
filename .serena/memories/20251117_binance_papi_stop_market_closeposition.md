## 2025-11-17 Binance PAPI STOP_MARKET closePosition
- `apps/rust-trading-bot/src/binance_client.rs::set_stop_loss` 中 STOP_MARKET 下单需要 `closePosition=true` 且不能传 `quantity`。
- 存量接口保留 `symbol/side/stopPrice/positionSide/workingType/timestamp`，调用方仍传入数量但在函数内部不再使用。