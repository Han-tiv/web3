## Gemini ETH analyzer 信号解析增强
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 现扩展操作关键词常量与 `matches_any_keyword`，支持限价/计划委托/平仓/撤单多语言与模糊表达。
- `extract_price` 增加多正则模板、货币符号与范围容错，并限制价格区间 (0, 1_000_000)。
- 新增限价单/计划委托执行前的入场价偏离检测及止盈止损合理性校验，避免解析异常误触发订单。
- 解析失败时输出提示与文本片段，便于排障。