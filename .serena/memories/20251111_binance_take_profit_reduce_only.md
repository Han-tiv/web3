## 2025-11-11
- apps/rust-trading-bot/src/binance_client.rs 的 set_limit_take_profit 不再在查询字符串中附加 reduceOnly=true，只保留 positionSide 以避免 -1106 错误。
- 修改后需确保 limit 单靠 positionSide 判定平仓，未来巡检其它订单方法是否仍硬编码 reduceOnly。