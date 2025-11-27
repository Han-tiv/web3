# Gemini ETH 分析器 Dry-Run 模式
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 现支持 `DRY_RUN=true` 环境变量，仅记录日志而不执行真实交易。
- `main` 会打印当前运行模式，并将 `dry_run` 传入 `analyze_eth_usdt` 和 `execute_trade_action`。
- `fulfill_pending_tpsl_orders` 与 `execute_trade_action` 在模拟模式下都会输出 `🧪 [DRY-RUN]` 日志并跳过真实 API 调用，同时保持风控检查逻辑一致。