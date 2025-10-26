# MCP 环境修复指南（2025-10-13，Codex）

## 背景
- Codex CLI 启动 MCP server 时出现 `No such file or directory (os error 2)` 与 `request timed out` 报错。
- 原因是 PATH 未正确指向 mise 管理的工具链，导致 `npx`、`uvx`、`mcp-proxy` 等命令不可见或超时。
- 目标是统一使用 mise 管理 Node.js 生态并校验所有 MCP server 的 CLI。

## 环境现况
- `mise` 版本：2025.9.23（提示可更新至 2025.10.8）。
- 全局工具：`python=3.13.7`（仓库 mise.toml），`node=22.20.0`（~/.config/mise/config.toml）。
- 关键命令：`node/npm/npx`、`uv/uvx`、`mcp-proxy`、`git` 已存在于 `~/.local/share/mise/shims` 与 `~/.local/bin`。
- MCP 服务清单（来自 `.codex/config.toml`）：
  - `npx`: context7、playwright、sequential-thinking、memory、shrimp-task-manager、chrome-devtools
  - `uvx`: duckduckgo-search、fetch、serena
  - `mcp-proxy`: deepwiki、microsoft-docs

## 修复流程
1. **同步 mise 工具链**
   ```bash
   mise install node@22.20.0
   mise use --global node@22.20.0
   mise reshim
   ```
   - 确认 `~/.config/mise/config.toml` 中 `node = "22.20.0"`。
2. **确保 PATH 包含 mise shims**
   - 临时：`export PATH="$HOME/.local/share/mise/shims:$PATH"`.
   - 永久：在 `~/.bashrc` 或 `~/.profile` 中追加 `eval "$(mise activate bash)"`。
3. **对齐现有符号链接（当前会话即时生效）**
   ```bash
   ln -sf ~/.local/share/mise/shims/{node,npm,npx} ~/.local/bin/
   ```
4. **保留 uv/uvx 与 mcp-proxy**
   - uv/uvx 已安装在 `~/.local/bin`，可选通过 mise pipx 或脚本安装。
   - `mcp-proxy` 由 `uv tool install mcp-proxy` 提供，位于 `~/.local/bin/mcp-proxy`。
5. **重新加载 Codex CLI**
   - 重启终端会话或运行 `codex mcp restart`（若 CLI 支持）。
   - 观察 `.codex/log/codex-tui.log` 是否仍出现 ENOENT/timeout。

## 验证命令
```bash
node -v
npm --version
npx --version
uvx --version
mcp-proxy --version
git --version

npx -y @upstash/context7-mcp --help
npx -y @playwright/mcp@latest --help
npx -y @modelcontextprotocol/server-sequential-thinking --help
npx -y @modelcontextprotocol/server-memory --help
npx -y mcp-shrimp-task-manager --help
uvx duckduckgo-mcp-server --help
uvx mcp-server-fetch --help
uvx --from git+https://github.com/oraios/serena serena start-mcp-server --context codex --help
npx -y chrome-devtools-mcp@latest --help
mcp-proxy --transport streamablehttp https://mcp.deepwiki.com/mcp --help
mcp-proxy --transport streamablehttp https://learn.microsoft.com/api/mcp --help
```

## 残余风险
- `chrome-devtools-mcp` 需要系统安装 Chrome/Chromium；首次运行可能触发浏览器下载。
- `playwright` 可能提示安装浏览器二进制，可按 CLI 提示执行 `npx playwright install`.
- 远程 MCP（deepwiki、microsoft-docs）依赖外部网络，网络阻塞将导致启动失败。
- Serena 通过 `uvx --from git+...` 实时克隆，首次启动需等待依赖安装。

## 建议
1. 定期执行 `mise self-update`、`mise upgrade node` 保持最新版本。
2. 在 shell 启动脚本中启用 `mise activate`，避免 PATH 再次丢失。
3. 记录验证结果至 `.codex/testing.md`、`.codex/verification.md`，便于回溯。
4. 若 MCP 启动仍失败，检查 `.codex/log/codex-tui.log` 并收集错误时间戳与命令行输出。
