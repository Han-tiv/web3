# 2025-11-09 web_server 状态接口热修
- apps/rust-trading-bot/src/web_server.rs 不再依赖数据库的 positions/equity 表：移除了 record_equity/upsert_position/remove_position，状态接口直接使用 exchange.get_positions() 来统计持仓，并以 Utc::now() 作为 last_update。
- 顶部数据库导入仅保留 AiAnalysisRecord/Database/TradeRecord，避免使用已删除的 PositionRecord、EquityRecord。