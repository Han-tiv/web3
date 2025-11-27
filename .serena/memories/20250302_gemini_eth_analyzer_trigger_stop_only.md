## 2025-03-02 Gemini ETH Analyzer Trigger 类型更新
- 文件 `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 中的 `ParsedAction::TriggerOrder` 分支现将触发单类型固定为 `STOP`。
- 背景：Binance 会将触发单类型为 `TAKE_PROFIT` 的开仓单误判为止盈单，导致下单失败。
- 处理：删除多空与价格比较的分支逻辑，统一注释说明“开仓永远用 STOP”。
- 影响：新增计划委托都以 STOP 类型下单，避免被误识别为止盈。