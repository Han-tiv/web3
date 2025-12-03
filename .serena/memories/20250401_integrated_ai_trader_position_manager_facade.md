# Integrated AI Trader Position Manager Facade
- 位置: `apps/rust-trading-bot/src/bin/integrated_ai_trader/core/position_manager.rs`
- 结构: `PositionManagerFacade` 占位门面, 目前只提供 `new()` 构造, 未来承接 `IntegratedAITrader` 的 `monitor_positions` 逻辑。
- 模块导出: `core/mod.rs` 现已导出 `position_manager` 模块及 `PositionManagerFacade`, 方便后续迁移。