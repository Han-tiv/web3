## 2025-12-05 分批持仓AI评估复用
- 文件: apps/rust-trading-bot/src/bin/integrated_ai_trader.rs
- 新增 evaluate_position_with_ai() 把原先持仓管理AI分析流程整合为公共方法。
- monitor_positions 中的 staged_manager 分支即使未触发硬性止损也调用该函数, 允许AI动态止盈。
- close_position_fully() 平仓成功后会同步清理 staged_manager 记录, 避免残留。