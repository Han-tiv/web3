## 2025-03-02 Gemini ETH Analyzer Trigger 分支
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 在 `execute_trade_action` 中新增 `ParsedAction::TriggerOrder` 分支。
- 分支对计划委托进行方向、触发价、止盈、止损等校验，并在余额充足时调用 `binance.place_trigger_order` 设置逐仓触发单。
- 触发类型依据方向和当前价格自动选择 `TAKE_PROFIT_MARKET` 或 `STOP_MARKET`，下单后暂不立即设置止盈止损，待成交后处理。