## Phase E-2: execute_recommended_actions 拆分
- 2025-04-03 新增 `execution/action_executor.rs`，封装 AI 推荐动作的限价单/触发单/止盈止损等执行分支，通过 `Weak<IntegratedAITrader>` 访问交易所、订单管理、触发单队列。
- `IntegratedAITrader` 增加 `action_executor` 字段，`execute_recommended_actions` 改为排序+调度器，调用 `ActionExecutor::execute_single_action` 完成一切动作。
- 止盈止损清理逻辑迁移到 `ActionExecutor::cancel_symbol_trigger_orders`，`trader.rs` 仅保留薄封装，文件降至 2282 行，满足 Phase E-2 行数指标。