## 2025-11-10
- `BinanceClient::limit_order` 现在接受 `reduce_only: bool` 参数，并在 `apps/rust-trading-bot/src/binance_client.rs` 中根据该值追加 `&reduceOnly=true` 到查询串。
- 所有开仓调用传入 `false`，`integrated_ai_trader.rs` 中的平仓调用传入 `true`，以防止意外反向开仓。
- 未来新增调用需显式决定 `reduce_only` 语义，避免默认行为不明确。