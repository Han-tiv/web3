# 触发单监控系统
- 在 `apps/rust-trading-bot/src/bin/integrated_ai_trader.rs` 新增 `TriggerOrderRecord` 结构、`active_trigger_orders` 字段以及 `monitor_trigger_orders`/`should_cancel_trigger_order`，实现触发单跟踪。
- `execute_recommended_actions` 在触发单下单成功后会写入跟踪列表，`CANCEL_TRIGGER` 操作会同步清理列表。
- `monitor_positions` 循环新增节拍计数器，每两个周期调用一次触发单监控，按需取消或移除已完成的触发单。