# 2025-11-13 延迟开仓队列后台重分析
- `IntegratedAITrader` 增加 `reanalyze_pending_entries` 循环线程，每 10 分钟扫描 `pending_entries`，超过 6 小时、已建仓或检测到资金出逃的币种会被移除。
- 线程会重新调用 `analyze_and_trade` 对仍符合条件的挂起币种做 AI 分析，并回写重试次数/时间。
- `main` 同步启动该线程，确保与 `monitor_positions` 并行运行，日志提示“延迟开仓队列重新分析线程已启动（每10分钟）”。