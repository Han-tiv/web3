# 2025-12-02 Gemini Client
- 新增 `apps/rust-trading-bot/src/gemini_client.rs`，封装 Google Gemini (`gemini-2.0-flash-exp`) AI 客户端。
- 复用 `deepseek_client` 中的 TradingSignal/Position 数据结构，只新增请求/响应模型与 Gemini API 封装。
- 模块已在 `src/lib.rs` 中通过 `pub mod gemini_client;` 暴露，调用方式与 DeepSeekClient 接口一致。