## 2025-03-23
- apps/rust-trading-bot/src/binance_client.rs 的多种下单函数（市价、限价、触发、止盈/止损、通用限价）已移除 positionSide 参数，仅依赖 side，符合 Binance 单向持仓模式。