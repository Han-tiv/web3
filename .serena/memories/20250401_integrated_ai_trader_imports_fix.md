# integrated_ai_trader 模块导入约定
- 自 2025-04-01 起，`apps/rust-trading-bot/src/bin/integrated_ai_trader/mod.rs` 直接声明 `pub mod core` 与 `pub mod modules`，并在根模块重新导出 `modules::config::*` / `modules::types::*`。
- `trader.rs` 及其子模块需通过 `super`/`super::super` 引用同级模块，禁止再使用 `crate::bin::integrated_ai_trader::*` 旧路径。
- 需要 `VolatilityCacheEntry`、`PositionAction` 等类型时，可直接从根模块（`super::super`）或 `modules::types` 导入，保持路径一致性以避免编译失败。