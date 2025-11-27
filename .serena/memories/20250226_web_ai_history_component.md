# Web 前端 AI 历史组件
- apps/rust-trading-bot/web/src/components/AiHistory.tsx 显示 AI 分析历史表格：时间、币种、分析类型、决策、置信度、原因，分页每页 10 条，自动按时间降序排序，刷新间隔 60 秒，并处理加载/错误/空状态。
- AiHistoryEntry 类型现包含 id/timestamp/symbol/analysis_type/decision/confidence/reason 字段并保留索引签名，置信度兼容 0-1 与 0-100 格式。