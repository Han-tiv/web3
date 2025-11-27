# EquityChart 刷新与真实数据
- Web 控制台 `apps/rust-trading-bot/web/src/components/EquityChart.tsx` 去除了本地 Mock 数据入口，统一通过 `api.getEquityHistory()` 请求真实权益曲线。
- 组件新增手动刷新按钮、30 秒轮询和加载/错误态提示，避免首次进入或手动刷新时界面无反馈。