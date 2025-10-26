# 测试与验证记录（2025-10-19，Codex）

## 手动验证步骤
1. `pgrep -f profit_monitor` → 未找到进程
2. `ps -eo pid,cmd | grep profit_monitor | grep -v grep` → 未找到进程
3. `tail -n 40 apps/rust-trading-bot/start_both.log` → 确认脚本成功启动两个进程（PID 762726/762731）
4. `tail -n 40 apps/rust-trading-bot/profit_monitor.log` → 日志显示启动成功
5. `tail -n 40 apps/rust-trading-bot/signal_trader.log` → 日志显示启动成功
6. `ps -p 762726 -o pid,cmd` → 进程不在运行
7. `ps aux | grep 'signal_trader'` → 未找到进程

## 结论
- `timeout 120s` 成功限制 `start_both.sh` 执行时间并返回后台 PID 762714。
- `profit_monitor` 与 `signal_trader` 在 120 秒内被启动，但随后不再存活，推测因 `timeout` 终止脚本时向进程组发送 SIGTERM，导致子进程也被结束。
- 若需长期运行，需调整策略（例如增加超时时间、移除 `timeout`、或在脚本中 disown 子进程）。

## 追加验证（2025-10-19 13:24）
1. `pgrep -fl profit_monitor` → 无输出，确认已停用
2. `pgrep -fl signal_trader` → 无输出，当前未运行
3. `ps -eo pid,cmd | grep signal_trader | grep -v grep` → 无匹配，验证守护配置尚未启用
4. 检查 `apps/rust-trading-bot/start_both.sh` → 仅包含启动 signal_trader 的逻辑
5. systemd/supervisor 配置文件均指向项目 release 二进制，输出日志至 `signal_trader.log`

结论：profit_monitor 已完全停止，signal_trader 可通过新脚本或守护配置启动，当前默认处于停机状态以防误交易。

## systemd 运行验证（2025-10-19 13:33）
1. `systemctl --user status signal_trader.service` → Active: active (running)，PID 763360
2. `tail -n 20 apps/rust-trading-bot/signal_trader.log` → 显示成功连接 Telegram 与 Binance
3. `journalctl --user -u signal_trader.service -n 5` → 仅包含启动日志，无异常
4. `systemctl --user restart signal_trader.service` → 先前执行成功，服务在 13:32 重启后保持运行

结论：systemd 用户级服务已启用并保持运行，start.sh 菜单 5/6 可复用相同命令管理进程。

## 系统级 systemd 验证（2025-10-19 13:43）
1. `sudo systemctl status signal_trader.service` → Active: active (running)，PID 763545
2. `tail -n 20 apps/rust-trading-bot/signal_trader.log` → 服务重启后日志保持正常
3. `sudo systemctl restart signal_trader.service` 未执行（避免中断），但 `enable --now` 已确认自动启动

结论：系统级守护生效，服务随机器启动保持运行，start.sh 菜单已切换为系统级命令。

## Nitter 关注归档工具验证（2025-10-20）
### 自动化命令
1. `npm run build`（工作目录：`apps/social-monitor/services/nitter`）→ TypeScript 成功编译。
2. `npm test` → 先执行 `tsc`，随后 `node test/filters.test.js` 与 `node test/archiveFormat.test.js` 均输出成功信息。
3. `npm run archive:following -- --limit-accounts 1 --tweet-limit 3 --delay 2000` → 首个账号因 nitter.net 返回 HTTP 429 未获取到推文，但脚本正常写入 `data/following/mirrorzk/tweets.json|md` 并记录异常。

### 结论
- 编译与测试流程通过，新建的 Markdown/JSON 格式化逻辑与参数过滤函数行为符合预期。
- 归档脚本可在限流情况下落盘空数据并保留错误信息；建议长时间运行时调高延迟或配置私有 Nitter 实例。

## 归档工具分页增强验证（2025-10-20）
### 自动化命令
1. `npm run build`（目录：`apps/social-monitor/services/nitter`）→ 新增分页逻辑编译通过。
2. `npm test` → 过滤规则测试与 `archiveFormat.test.js`（校验分页/尝试信息）均成功。

### 结论
- 新版 `archiveFollowing` 支持分页游标、状态文件与 skip-existing 功能，相关单元测试覆盖 Markdown 元数据。
- 下一步需在本地 Nitter 实例上执行实际抓取验证（全量运行见后续记录）。

## happy-coder ripgrep 验证（2025-10-20 22:05）
1. `node scripts/unpack-tools.cjs`（工作目录：`~/.local/share/mise/installs/node/24.9.0/lib/node_modules/happy-coder`）→ 输出“Unpacking tools for x64-linux...”
2. `file tools/unpacked/ripgrep.node` → `ELF 64-bit LSB shared object, x86-64`
3. `file tools/unpacked/rg` → `ELF 64-bit LSB pie executable, x86-64`
4. `node scripts/ripgrep_launcher.cjs '["rg","--version"]'` → 输出 `ripgrep 14.1.1` 版本信息
5. `node scripts/ripgrep_launcher.cjs '["rg","-n","--max-count","5","nitter","apps/social-monitor/services/nitter"]'`（工作目录 `/home/hanins/code`）→ 0.1s 内返回匹配结果，ripgrep 不再卡住

结论：happy-coder 使用的 ripgrep 已重新解压为 Linux 版本，命令快速返回且 launcher 可正常加载，确认修复生效。

## MCP 预热脚本验证（2025-10-20 23:24）
1. `scripts/prewarm-mcp.sh` → 输出“所有 MCP 服务已预热完成，可启动 Codex CLI。”，无报错。
2. `tail -n 5 .codex/operations-log.md` → 可见脚本及各预热命令已留痕。

结论：统一预热脚本可顺利完成全部依赖的加载，执行时间约 20 秒。建议在启动 Codex CLI 前执行一次，确保后续 MCP 服务不会因首次下载耗时而超时。
