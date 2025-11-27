# Telegram连接健康监控
- 2025-11-13：IntegratedAITrader 新增 Telegram 健康监控机制。
- 结构体加入 `telegram_error_count` 与 `last_successful_message`，在消息循环里记录连续错误并恢复时重置。
- main() 启动 `monitor_telegram_health` 线程，方法每 5 分钟检查，超过 10 分钟无成功消息且持续错误会发出重启警告。