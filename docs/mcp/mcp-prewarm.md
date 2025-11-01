# Codex MCP 预热手册
- 日期：2025-10-20
- 执行者：Codex

## 背景
Codex CLI 启动时会按照 `~/.codex/config.toml` 并行拉起多个 MCP 服务。若首次执行时 npm/uvx 需要下载依赖，常会在握手阶段超时（日志中出现 `request timed out` 或 `No such file or directory`）。为避免重复故障，本次新增统一的预热脚本。

## 修复措施
1. 新增 `scripts/prewarm-mcp.sh`，串行调用以下命令以预加载依赖：
   - `npx`：sequential-thinking、memory、context7、mcp-shrimp-task-manager、playwright、chrome-devtools
   - `uvx`：duckduckgo-mcp-server、mcp-server-fetch、serena
   - `mcp-proxy`：验证版本
   - `curl`：探测 deepwiki 与 microsoft-docs 端点
2. 运行脚本后，相关包会缓存到本地，Codex CLI 随后即可在超时时间内完成启动。

## 使用步骤
```bash
cd /home/hanins/code
scripts/prewarm-mcp.sh
```

脚本会在终端打印进度并在完成时输出“所有 MCP 服务已预热完成，可启动 Codex CLI。”如果缺少必要命令（如 `uvx`），脚本会直接报错提示安装。

## 验证记录
- `.codex/operations-log.md`：记录了各 MCP 命令的 `--help` 冒烟测试与脚本执行情况。
- `scripts/prewarm-mcp.sh`：可重复执行，无副作用。
- 推荐在运行 Codex CLI 或 happy-coder 前执行一次预热；若包版本更新，可再次运行刷新缓存。

## 后续建议
- 将脚本纳入工作流，例如在启动前的 shell profile 中调用。
- 定期检查远程端点（deepwiki、microsoft-docs），若出现网络故障，可在脚本中加入重试或备用节点。
