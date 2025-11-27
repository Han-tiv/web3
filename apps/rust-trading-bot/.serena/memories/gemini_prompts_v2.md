Gemini 客户端已接入 Valuescan V2 方案：
- 在 src/gemini_client.rs 中新增 analyze_market_v2/analyze_position_management_v2，解析 valuescan_v2::{TradingSignalV2, PositionManagementDecisionV2}。
- 新增 build_entry_analysis_prompt_v2 与 build_position_management_prompt_v2，引用 AI_PROMPTS_V2.md 的最新 prompt 文案。