# Gemini ETH 分析器手续费与杠杆调整
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 将默认杠杆常量从 50 调整为 20。
- 动态本金上下限 `MIN_DYNAMIC_CAPITAL`/`MAX_DYNAMIC_CAPITAL` 均改为 0.5 USDT，以确保每笔固定 0.5 USDT 实际敞口。
- 目的：降低实盘爆仓风险，使 Gemini ETH 策略在极端波动下也只承受 10 USDT 不到的权益波动。