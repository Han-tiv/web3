## 2025-11-05 Binance PAPI STOP_MARKET 修复
- 在 `apps/rust-trading-bot/src/binance_client.rs` 的 STOP_MARKET 下单请求移除 `closePosition=false` 参数，避免与 `quantity` 同时存在导致的 `-1116 Invalid orderType` 错误。
- 该接口仍保留 `symbol/side/stopPrice/quantity/positionSide/workingType/timestamp` 参数，满足 PAPI 要求。