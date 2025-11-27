# Web 服务器 AppState 依赖
- integrated_ai_trader 主流程在启动 Web 服务器时，需要将 `trader.exchange.clone()` 传入 `web_server::AppState::new`，确保 Web API 可以复用实时交易所客户端。
- 文件：apps/rust-trading-bot/src/bin/integrated_ai_trader.rs（main 函数 Web 服务初始化段）。