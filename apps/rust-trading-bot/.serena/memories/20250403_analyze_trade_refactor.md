## Phase E: analyze_and_trade 拆分
- 2025-04-03 将 `analyze_and_trade` 的 K线抓取、入场区分析、AI 决策逻辑抽离到 `ai/kline_fetcher.rs`、`ai/entry_analyzer.rs`、`ai/ai_decider.rs`，`IntegratedAITrader` 通过新字段统一调度。
- `trader_entry_executor.rs` 负责试探建仓执行，`trader.rs` 保留调度、数据库记录与仓位管理，重复函数/方法已清理，文件约 2.68K 行。
- 关键组件：`KlineFetcher::fetch_multi_timeframe` 并发抓 5m/15m/1h，`EntryAnalyzer::analyze_entry_zones` 输出 1h/15m 区域，`AIDecider::make_trading_decision` 统一 Valuescan V1/V2。