## 2025-02-16 集成AI Trader仓位与杠杆调整
- 将 `IntegratedAITrader` 的动态仓位范围更新为 1-2 USDT，杠杆范围更新为 6-10x
- 初始化配置及置信度注释同步更新，新的名义价值示例分别为 20U (高)、12U (中)、6U (低)
- 变更位置：`apps/rust-trading-bot/src/bin/integrated_ai_trader.rs`