# 2025-11-09 数据库裁剪
- apps/rust-trading-bot/src/database.rs 已删除 positions 与 equity_history 表（建表 SQL、所有增删改查函数、计数接口）。
- PositionRecord、EquityRecord 结构体及相关 map_* 函数、测试一并移除，仅保留 trades 与 ai_analysis 数据面。
- 未来若需要展示持仓/权益数据，需要依赖其他服务或重新设计表结构。