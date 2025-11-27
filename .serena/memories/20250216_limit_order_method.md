## 2025-02-16
- 在 `apps/rust-trading-bot/src/binance_client.rs` 中为 `BinanceClient` 新增 `limit_order` 方法，按 PAPI `/papi/v1/um/order` LIMIT + GTC 接口下单。
- 方法支持 `symbol`、`quantity`、`side` (BUY/SELL)、`limit_price` 以及可选 `positionSide`，沿用 `set_limit_take_profit` 的请求签名与精度处理流程。
- 未来如需新增其它下单类型，可复用该模板并注意精度格式化与签名顺序。