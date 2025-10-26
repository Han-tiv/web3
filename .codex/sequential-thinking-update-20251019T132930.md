# 手动 sequential-thinking 记录
- 时间：2025-10-19T13:29:30+08:00
- 任务：将 signal_trader 以 systemd 守护运行，并通过 start.sh 控制启动/停止

## 任务理解
1. 需要真正安装并启用 systemd service，使 signal_trader 作为后台服务运行。
2. start.sh 应成为管理入口，提供启动/停止/状态等操作，避免手动 systemctl 命令。
3. 需要验证服务实际运行情况，并记录日志。

## 技术方案评估
- 使用现成的 `apps/rust-trading-bot/systemd/signal_trader.service` 复制到 `/etc/systemd/system/`。
- 运行 `systemctl daemon-reload`、`systemctl enable --now signal_trader`，并确认状态。
- 修改 `start.sh` 增加新菜单项：例如启动 systemd 服务、停止服务、查看状态、查看日志。
- start.sh 运行时可检测 systemctl 是否可用；处理无权限情况。

## 风险识别
- systemd 需要 sudo 权限；确认当前环境是否允许 `sudo`.
- `.env` 若存在不兼容语法会导致服务启动失败，需在日志中检查。
- start.sh 增加 systemctl 操作需避免影响原有功能。

## 实现步骤规划
1. `sudo cp` 服务文件并 `systemctl daemon-reload`.
2. `sudo systemctl enable --now signal_trader`.
3. `systemctl status signal_trader` 记录输出。
4. 修改 `start.sh` 添加 systemd 管理菜单。
5. 更新 `DAEMON_SETUP.md` 等文档说明 start.sh 使用。
6. 更新测试与验证记录。

## 边界条件分析
- 若 systemctl 命令失败，应记录并提示用户手动处理。
- start.sh 需兼容非交互执行? 目前 interactive 选项 ok。
- 确保 signal_trader.log 路径可写。
