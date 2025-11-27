# DeepSeek 持仓管理 Prompt 止盈止损强制要求 (2025-11-11)
- 文件 `apps/rust-trading-bot/src/deepseek_client.rs` 的 `build_position_management_prompt` 更新了【输出要求】。
- 顶层 JSON 字段 `take_profit`、`stop_loss` 现在必须输出具体价位，禁止返回 null。
- 新增提示：无论是否已有保护单都要给出建议，并强制在 `recommended_actions` 中包含 `SET_STOP_LOSS_TAKE_PROFIT`（需说明新建或调整，若现有价位不合理要提出调整）。