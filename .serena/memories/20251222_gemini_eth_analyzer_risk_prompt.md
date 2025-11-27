# Gemini ETH 分析 Prompt 风控说明
- build_analysis_prompt 开头现包含固定的资金配置与风控要求段落，强调 2 USDT 逐仓、50 倍杠杆及止盈必须大于止损。
- 更新引用：apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs 中 format! 的 Raw string 文本，如需再改 prompt，记得保留该风险说明。