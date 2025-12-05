# DeepSeek 模型升级
- 时间：2025-03-30
- 位置：`apps/rust-trading-bot/src/ai_core/deepseek/mod.rs`
- 更新内容：`DeepSeekClient` 全部请求（analyze_market、analyze_market_v2、analyze_position_v2、analyze_entry、analyze_entry_v3）统一使用模型 `deepseek-ai/DeepSeek-V3.2-Exp`，替代旧的 `deepseek-chat` 配置。
- 影响：所有调用 DeepSeek 的交易/持仓分析将命中 V3.2-Exp 模型，以保证策略分析一致性。