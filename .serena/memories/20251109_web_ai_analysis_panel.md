# Web 前端 AIAnalysisPanel
- 新增 apps/rust-trading-bot/web/src/components/AIAnalysisPanel.tsx。
- 组件从 /api/ai-history 获取 AI 分析记录，支持按币种筛选、彩色信号标签、置信度进度条以及分析内容展开/收起。
- 时间使用本地格式化，数据默认按最新时间倒序显示，并复用 Tailwind Binance 风格。