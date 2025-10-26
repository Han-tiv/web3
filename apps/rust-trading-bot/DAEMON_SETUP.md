# signal_trader 守护进程部署指南（更新于 2025-10-19，执行者：Codex）

本指南说明如何使用 systemd 或 Supervisor 管理 `signal_trader`，确保不再依赖 `profit_monitor` 并避免 `timeout` 信号导致的连带终止。

## 前置准备
- 仓库路径：`/home/hanins/code`
- 需要先执行 `cargo build --release --bin signal_trader`
- 确认 `.env` 配置完整且位于仓库根目录

## systemd 部署（推荐方案：系统级）

1. 复制单元文件：
   ```bash
   echo 'hanzhikun' | sudo -S cp apps/rust-trading-bot/systemd/signal_trader.service /etc/systemd/system/signal_trader.service
   ```
2. 重新加载并启用：
   ```bash
   echo 'hanzhikun' | sudo -S systemctl daemon-reload
   echo 'hanzhikun' | sudo -S systemctl enable --now signal_trader.service
   ```
3. 查看状态与日志：
   ```bash
   sudo systemctl status signal_trader.service --no-pager
   tail -f /home/hanins/code/apps/rust-trading-bot/signal_trader.log
   ```

> 单元文件以 `User=hanins`/`Group=hanins` 运行进程，并通过 `/bin/bash -lc 'set -a && source .env ...'` 加载根目录 `.env`，兼容行内注释；`Restart=on-failure` 会在异常退出时自动重启。

### 可选：继续使用用户级 systemd
若仍需用户级方式，可将单元复制到 `~/.config/systemd/user/` 并使用 `systemctl --user` 操作。

## Supervisor 部署
1. 复制配置：
   ```bash
   sudo cp apps/rust-trading-bot/supervisor/signal_trader.conf /etc/supervisor/conf.d/
   ```
2. 重载并启动：
   ```bash
   sudo supervisorctl reread
   sudo supervisorctl update
   sudo supervisorctl start signal_trader
   ```
3. 查看状态：
   ```bash
   supervisorctl status signal_trader
   tail -f /home/hanins/code/apps/rust-trading-bot/signal_trader.log
   ```

> Supervisor 配置使用 `bash -lc` 加载 `.env`，并启用 `stopasgroup`/`killasgroup` 保证信号同步到子进程。

## 手动脚本
仍需保留人工干预时，可以执行：
```bash
cd apps/rust-trading-bot
./start_both.sh
```
该脚本现仅启动 `signal_trader`，不再运行 `profit_monitor`，并会在 5 秒内完成自检后退出。

若希望通过交互式脚本管理 systemd，可使用同目录的 `start.sh`，菜单 5/6/7/8 分别对应 **启动 / 停止 / 状态 / 日志**，内部命令等同于 `sudo systemctl ...`。如需免密码，可自行配置 sudoers。

## 注意事项
- 如果要彻底删除 `profit_monitor`，可清理其日志与二进制：`rm -f profit_monitor.log target/release/profit_monitor`
- systemd 环境文件需要保持 `KEY=VALUE` 格式，不支持 `export`、引号或空格分隔的语法。
- Supervisor 与 systemd 二选一即可，避免重复启动同一进程。
