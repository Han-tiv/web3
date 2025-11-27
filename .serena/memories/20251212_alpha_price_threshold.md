# Alpha 币种价格阈值
- integrated_ai_trader.rs 中新增 Alpha 币种价格上限：价格>=1000 USDT 时跳过，<1000 才允许交易。
- 非 Alpha 币种继续沿用 100 USDT 的跳过阈值，日志描述保持一致。