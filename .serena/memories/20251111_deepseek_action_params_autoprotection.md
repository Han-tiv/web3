# DeepSeek ActionParams auto_set_protection
- 日期：2025-11-11
- 文件：apps/rust-trading-bot/src/deepseek_client.rs
- 更新：ActionParams 增加 `auto_set_protection: bool` 字段并默认 false（serde default），提示 AI 在开仓后是否需要自动设置保护单。
- Prompt：在持仓管理 JSON 说明与示例中新增该字段，要求 LIMIT/TRIGGER 等开仓动作给出 true/false，用于让执行器判断是否自动同步止损/止盈。