# 集成AI交易器硬性止损更新
- 在 `integrated_ai_trader.rs` 分批持仓逻辑中，于 AI 评估前新增 30min/60min/极端亏损硬性止损判定。
- 满足任一条件会直接调用 `close_position_fully` 平仓并移除 `staged_manager` 记录，避免 AI HOLD 覆盖风控。
- 检查顺序：30分钟-1.5%、60分钟-2%、60分钟未盈利、极端-5%。