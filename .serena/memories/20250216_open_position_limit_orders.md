## 2025-02-16
- `BinanceClient` 的 `open_long`/`open_short` 现已改为调用 `limit_order`，限价为当前价乘以 1.001/0.999 并传入 positionSide（LONG/SHORT）。
- 未来如需调整开仓方式，需同步考虑限价因子与 positionSide 传参。