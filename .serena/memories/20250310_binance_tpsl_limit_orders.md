# Binance TPSL 限价化调整
- 为满足交易所策略要求，`BinanceClient::set_stop_loss` 与 `set_take_profit` 改为限价触发：需要 `limit_price: Option<f64>` 参数，并默认使用 `stop_price`。
- 所有调用点（Gemini ETH Analyzer、Integrated AI Trader、copy_trader 等）需传入 `None` 或具体限价，且触发单回调需同步更新。
- Gemini ETH Analyzer 的触发单和止盈止损均已改用 STOP/TAKE_PROFIT 限价触发，`place_trigger_order` 传入 `Some(trigger_price)`。