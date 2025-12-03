## Phase B: monitor_positions 拆分
- 2025-04-02 将 `monitor_positions` (集成AI交易主循环) 拆分成 4 个 async 子方法：`monitor_trial_positions`、`monitor_staged_stop_loss`、`execute_position_protection`、`batch_evaluate_positions`，均位于 `trader.rs` 内并完整保留原有业务逻辑。
- 新增模块级 `TrackerSnapshot` 结构体，在 `position_trackers` 快照构建时使用；`monitor_positions` 现在只负责任务调度与调用子方法，主体约 120 行。
- 未来若继续瘦身 `trader.rs`，可以进一步将上述子方法迁移到 `src/bin/integrated_ai_trader/core/` 等子模块。