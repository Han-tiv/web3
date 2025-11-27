# Integrated AI Trader 初始余额策略
- 2025-12-02：integrated_ai_trader 主流程中不再调用 Binance API 获取 totalMarginBalance。
- initial_balance 现为硬编码 12.90 USDT，并在日志中提示“初始合约余额（固定）”。
- Web 监控状态等下游模块应继续使用该固定值，盈亏计算从 12.90 USDT 起算。