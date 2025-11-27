## 2025-11-05 Alpha消息格式修复部署
- 按用户指令kill旧进程(19231)，在apps/rust-trading-bot执行`cargo build --bin integrated_ai_trader --release`成功
- 使用`nohup`以`RUST_LOG=info`启动最新`integrated_ai_trader`，进程PID 35128
- 最新日志显示Telegram/Binance初始化完成，Alpha/FOMO监控线程已运行