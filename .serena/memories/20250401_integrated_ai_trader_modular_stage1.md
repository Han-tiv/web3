# Integrated AI Trader 模块化 Stage 1
- trader.rs 新增模块化组件字段（SignalProcessor、RiskController、EntryManager、OrderExecutor、PositionCloser、TriggerMonitor），构造函数改为返回 `Result` 并一次性初始化 TrackerManager/CacheManager/HistoryRecorder。
- active_trigger_orders 改为 `Arc<RwLock<HashMap<...>>>`，last_analysis_time 改用 `Instant`，信号清理委托给 SignalProcessor。
- 新增 `core/signal_processor.rs` 与 `core/risk_controller.rs`，在 `core/mod.rs` 里统一导出。