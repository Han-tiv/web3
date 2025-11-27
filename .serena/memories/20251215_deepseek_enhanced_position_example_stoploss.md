调整 `apps/rust-trading-bot/src/deepseek_client.rs` 中 EnhancedPositionAnalysis 的输出示例：
- 示例 stop_loss/take_profit 改为具体价位并强调可从旧价位调整至新值。
- recommended_actions 展示 SET_STOP_LOSS_TAKE_PROFIT 如何调整已有保护单。
- 新增注释，要求 AI 检查现有止盈止损并在 recommended_actions 写明旧值与新值。