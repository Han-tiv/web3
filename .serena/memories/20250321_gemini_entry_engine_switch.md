# 2025-03-21 Gemini 入场引擎切换
- 集成 AI 交易器 EntryManager 与 AIDecider 全部调用 `GeminiClient::{analyze_market, analyze_market_v2}`，不再依赖 DeepSeek 进行开仓分析。
- `IntegratedAITrader::new` 以及配置加载逻辑只需要 `GEMINI_API_KEY`，`DEEPSEEK_API_KEY` 被移除。
- 启动日志更新为 "Gemini(入场分析) + Gemini(持仓管理)", 文档亦同步强调 Gemini 双引擎配置。