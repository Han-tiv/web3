# Gemini ETH-USDT Analyzer
- 新增 bin 程序 `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs`，周期性拉取 Binance K 线/仓位数据并调用 GeminiClient 生成分析。
- 依赖环境变量：BINANCE_API_KEY、BINANCE_SECRET、GEMINI_API_KEY (或 GOOGLE_GEMINI_API_KEY)。
- 包含技术指标计算、止盈止损与计划委托收集、构建详细 prompt 后打印 Gemini 返回结果。