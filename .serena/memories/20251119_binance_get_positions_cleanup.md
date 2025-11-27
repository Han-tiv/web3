## Binance 客户端持仓查询清理
- 2025-11-19 删除 `apps/rust-trading-bot/src/binance_client.rs` 中基础 `impl BinanceClient` 下的旧 `get_positions`，避免覆盖 `ExchangeClient` trait 实现。
- 现在仅保留 trait 版本以支持数组与 map 等多种返回格式。