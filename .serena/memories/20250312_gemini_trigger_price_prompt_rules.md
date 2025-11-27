## 2025-03-12 Gemini 触发单方向校验与 Prompt
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 在 `ParsedAction::TriggerOrder` 分支新增了触发价方向校验，做空触发价必须低于当前价，避免 Binance -2021 错误。
- `build_analysis_prompt` 的“操作3: 合约计划委托下单”段落新增详细触发价规则与示例，强调做空触发价 < 当前价，以及做多/做空止盈止损的价格关系。