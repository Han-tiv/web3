## 2025-11-11
- 在 `apps/rust-trading-bot/src/deepseek_client.rs` 中新增 `DeepSeekClient::evaluate_positions_batch`，支持批量持仓评估。
- 新方法重用 `build_batch_evaluation_prompt`，调用 DeepSeek Chat API，并解析/校验 `BatchDecisionResponse`，将结果映射为 `(symbol, PositionManagementDecision)`。
- 增强了 DeepSeek 客户端在 `integrated_ai_trader` 等流程中一次性获取多仓位 AI 决策的能力。