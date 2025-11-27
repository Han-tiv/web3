# 2025-12-02 Grok 客户端
- 新增 `src/grok_client.rs`，完整复刻 DeepSeek AI 交易模块的接口与 build_* 辅助方法。
- Grok 客户端复用 deepseek_client 中的 TradingSignal、PositionManagementDecision、Kline 等数据结构（通过 pub use），仅调整结构体和 API 请求以对接 Grok。
- Grok API 端点：https://api.x.ai/v1/chat/completions，模型 `grok-2-1212`，OpenAI 兼容请求格式。