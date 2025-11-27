# Integrated AI Trader 调度参数
- `apps/rust-trading-bot/src/bin/integrated_ai_trader.rs` 引入 `const POSITION_CHECK_INTERVAL_SECS: u64 = 180`，用于控制持仓监控线程的 AI 评估节奏（默认 3 分钟，因止盈止损已启用，频率可降低）。
- 新增 `const USE_ENHANCED_ANALYSIS: bool = false` 作为增强分析开关，未来用于在 `analyze_position_management` 与增强版分析之间切换。