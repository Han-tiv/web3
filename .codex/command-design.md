# 后台执行命令设计
- 时间：2025-10-19T13:06:30+08:00
- 目标：在加载根目录 `.env` 后，以后台方式运行 `start_both.sh`，并加入 `timeout 120s` 防止脚本长时间占用终端。

## 最终命令
```bash
cd apps/rust-trading-bot && \
set -a && source ../../.env && set +a && \
nohup bash -lc 'timeout 120s ./start_both.sh' > start_both.log 2>&1 & echo $!
```

## 设计说明
1. `set -a` + `source ../../.env` + `set +a`：复用用户原有模式，为 `start_both.sh` 与其子进程显式提供环境变量（尽管脚本内部会再次加载 .env）。
2. `nohup bash -lc 'timeout 120s ./start_both.sh'`：使用子 shell 保证 `timeout` 与脚本逻辑在同一进程组中执行；`nohup` 确保会话断开后仍在后台运行。
3. `timeout 120s`：120 秒后向 `start_both.sh` 发送 `SIGTERM`，脚本退出。由于脚本内部已经使用 `nohup` 启动 `profit_monitor` 与 `signal_trader`，二者将继续运行。
4. `> start_both.log 2>&1`：单独记录脚本输出，便于排查初始化问题。
5. `& echo $!`：返回后台 `nohup` 进程 PID，方便后续追踪。需注意该 PID 在 120 秒超时后会退出，子进程 PID 以脚本日志为准。

## 风险提醒
- `timeout` 会提前终止脚本的持续监控循环，如需长期监控需改造脚本逻辑或延长超时时间。
- 若 `.env` 缺少必需变量，脚本将在 120 秒内失败并记录到 `start_both.log`。
