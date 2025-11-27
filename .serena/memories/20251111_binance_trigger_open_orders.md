# Binance 触发开仓单实现
- 在 `apps/rust-trading-bot/src/binance_client.rs` 中新增 `place_trigger_order`，支持 STOP/STOP_MARKET/TAKE_PROFIT/TAKE_PROFIT_MARKET 条件单，可选择 OPEN/CLOSE、LONG/SHORT，两侧自动映射 BUY/SELL。
- 统一按交易规则格式化 quantity/stop/limit 价格，缺失 limit_price 的 STOP/TAKE_PROFIT 会报错。
- 请求参数包含 `workingType=MARK_PRICE`，成功/失败日志分别输出 🎯 提示或完整错误体。