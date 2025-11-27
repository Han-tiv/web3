## DeepSeek JSON 解析增强
- 对 `apps/rust-trading-bot/src/deepseek_client.rs` 中 `analyze_market` 与 `analyze_position_management` 方法新增 `info!` 调试日志，打印 AI 原始 JSON 响应。
- 将 `serde_json::from_str` 调整为 `match` 结构，失败时使用 `error!` 输出解析错误及原始内容，并通过 `anyhow::bail!` 返回详细错误。
- 方便在 DeepSeek 返回非标准 JSON 时快速定位问题。