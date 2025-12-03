### 2025-04-04 Clippy 常规告警清理
- 移除了多处 `as i64` 冗余转换、`post(&format!(..))` 的多余取引用、以及 `.get(0)` 访问的旧写法（现改用 `first()`）。
- 针对 bitget/okx/trial monitor 等 identical-if 结构做了简化；去掉了 `debug_unified`/`check_balance` 系列脚本里多余的 `use reqwest;`。
- 修复 `support_analyzer.rs` 中 `vec![...]` 的 `useless_vec` 告警、`trader_entry_executor.rs` & `entry_manager.rs` 中重复字段名、以及 `smart_money_trader` 未使用参数（前缀 `_`）。
- 当前 `cargo clippy 2>&1 | grep "warning:" | wc -l` ≈ 59，后续可继续处理更复杂告警。