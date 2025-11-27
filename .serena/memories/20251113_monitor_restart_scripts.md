# Telegram 监控与重启脚本
- 在 apps/rust-trading-bot 下新增 monitor_and_restart.sh 与 start_monitor.sh。
- monitor_and_restart.sh 轮询 integrated_ai_trader.log，检测 Telegram 错误/断线并自动 stop.sh+start_trader.sh 重启。
- start_monitor.sh 负责 nohup 后台启动监控脚本，输出到 monitor.log，并避免重复实例。
- 两个脚本已赋予可执行权限，便于直接运行。