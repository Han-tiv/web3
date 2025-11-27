## 2025-02-24 平仓交易记录落地
- 在 `apps/rust-trading-bot/src/bin/integrated_ai_trader.rs` 内, `close_position_fully` 平仓成功后会构造 `web_server::TradeRecord` 并调用 `web_state.add_trade`。
- 记录所需的数据优先来自 `PositionTracker`, 如果缺失会退回 `StagedPositionManager` 的快照; 时间戳不足时使用内部 `timestamp_ms_to_datetime` 转换。
- TradeRecord 包含开仓/平仓价、数量、盈亏、盈亏百分比、入场/出场时间以及持仓时长, 便于 Web 控制台展示历史交易。