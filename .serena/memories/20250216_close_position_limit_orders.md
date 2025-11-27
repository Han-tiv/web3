## 2025-02-16
- `apps/rust-trading-bot/src/bin/integrated_ai_trader.rs` 中 `close_position_fully` 与 `close_position_partially` 改为调用 `BinanceClient::limit_order`，平多使用 当前价×0.999 的 SELL 限价，平空使用 当前价×1.001 的 BUY 限价。
- 下单时带上 `positionSide`，记录返回的 `orderId` 并写入日志，保持与此前开仓限价方式一致。