## Phase C+D: monitor_positions 模块化
- 2025-04-02 将原先集中在 `trader.rs` 的 `monitor_trial_positions`、`monitor_staged_stop_loss`、`execute_position_protection`、`batch_evaluate_positions` 迁移到 `execution/` 目录，分别对应 `TrialPositionMonitor`、`StagedStopLossMonitor`、`PositionProtector`、`BatchEvaluator`，通过 `Weak<IntegratedAITrader>` 调用核心状态与辅助方法。
- `IntegratedAITrader` 新增四个组件字段，并在 `monitor_positions` 中顺序调用，`new()` 现在返回 `Arc<Self>` 并使用 `Arc::new_cyclic` 初始化含自引用组件。
- `tracker.rs` 现仅保留计数调度与快照构造逻辑，整体行数缩减至约 3.2K 行，为后续 execution 模块扩展铺路。