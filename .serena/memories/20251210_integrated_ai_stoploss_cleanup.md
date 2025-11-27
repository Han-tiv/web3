# 集成AI交易器止损规则调整
- 文件：`apps/rust-trading-bot/src/bin/integrated_ai_trader.rs`。
- 删除 30min -1.5%、60min -2%/未盈利，以及 4 小时未盈利三个硬编码止损条件。
- 当前仅保留 -5% 极端止损后再进入 AI 动态评估。
- 变更影响 staged manager 的强制平仓逻辑，后续 Prompt 无需同步更新。