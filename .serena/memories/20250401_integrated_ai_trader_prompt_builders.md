# Integrated AI Trader prompt上下文
- 2025-04-01 在 `apps/rust-trading-bot/src/bin/integrated_ai_trader/modules/types.rs` 内新增 `EntryPromptContext<'a>`，包含 alert 详情、k线引用与可选技术指标，并通过 `pub use` 重新导出 `TechnicalIndicators`，供模块树使用。
- 同一文件给 `PreparedPositionContext` 实现 `to_prompt_context()`，用于生成 AI prompt 文本。
- `apps/rust-trading-bot/src/bin/integrated_ai_trader/trader.rs` 新增 `build_entry_prompt_v1/v2` 以及 `build_position_prompt_v2`，复用 `EntryPromptContext` 与 `PreparedPositionContext` 来生成 Gemini/DeepSeek 所需 prompt。