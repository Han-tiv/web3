## DeepSeek 客户端调试日志更新
- 在 `apps/rust-trading-bot/src/deepseek_client.rs` 中为 `analyze_market` 和 `analyze_position_management` 方法加入调试日志。
- 新日志会在解析 DeepSeek 返回 JSON 时输出原始 `content` 内容，并在解析失败时打印错误及原文，便于排查 AI 响应导致的解析问题。
- 保持 `log` 宏使用（`info!`/`warn!`），无额外依赖调整。