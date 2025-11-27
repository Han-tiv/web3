# rust-trading-bot equity chart interval
- 日期：2025-11-11
- 文件：apps/rust-trading-bot/src/web_server.rs
- 变更：维持12个采样点，将 `INTERVAL_MINUTES` 从 5 提升到 60，并在常量处加注释，确保网页端权益曲线显示过去12小时跨度。