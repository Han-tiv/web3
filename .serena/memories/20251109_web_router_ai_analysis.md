# Web 前端路由更新
- apps/rust-trading-bot/web/src/App.tsx 现已引入 react-router-dom，并提供 '/' 与 '/ai-analysis' 两个视图。
- 头部导航新增“AI分析”(🤖)菜单项，所有页面共享统一布局。
- AIAnalysisPanel 通过 /ai-analysis 路由访问，Dashboard 默认在根路径并保留 BackendStatus、EquityChart、PositionsList、TradesHistory。