# Web Server Equity Chart Interval 调整
- 日期：2025-11-11
- 位置：apps/rust-trading-bot/src/web_server.rs，dashboard equity 序列常量
- 修改：保持 STEPS=12，将 INTERVAL_MINUTES 从 60 调整为 15，对应 3 小时跨度
- 影响：前端 /status equity 折线数据分辨率提升，时间窗缩短至 3 小时，便于近实时监控