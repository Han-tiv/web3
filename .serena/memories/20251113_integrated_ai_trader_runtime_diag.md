# 2025-11-13 集成交易机器人巡检
- 进程 `./target/release/integrated_ai_trader` 正在运行 (示例 PID 118319)。
- 06:35Z 之后 Telegram 持续 read error 重连，日志连续刷 `[ERROR] Telegram连接错误`。
- 最近 2 小时数据库无新交易，持仓仍为 DOGE/ZEC/MERL/BNB 四笔小额多单，持仓 89h+。
- 日志最近 200 行无延迟队列、Telegram 信号、AI评估/止损相关输出，表现为信号/执行停滞。