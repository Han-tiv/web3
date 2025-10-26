#!/usr/bin/env bash
set -euo pipefail

# 预热 Codex 所需的 MCP 服务，避免首次启动时因 npm/uvx 下载超时

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "缺少命令：$1，请先安装后再运行本脚本。" >&2
    exit 1
  fi
}

run_step() {
  echo ">>> $*"
  "$@"
}

require_cmd npx
require_cmd uvx
require_cmd mcp-proxy
require_cmd curl

echo "==> 预热 npx MCP 服务"
run_step npx -y @modelcontextprotocol/server-sequential-thinking --help >/dev/null
run_step npx -y @modelcontextprotocol/server-memory --help >/dev/null
run_step npx -y @upstash/context7-mcp --help >/dev/null
run_step npx -y mcp-shrimp-task-manager --help >/dev/null
run_step npx -y @playwright/mcp@latest --help >/dev/null
run_step npx -y chrome-devtools-mcp@latest --help >/dev/null

echo "==> 预热 uvx MCP 服务"
run_step uvx duckduckgo-mcp-server --help >/dev/null
run_step uvx mcp-server-fetch --help >/dev/null
run_step uvx --from git+https://github.com/oraios/serena serena start-mcp-server --context codex --help >/dev/null

echo "==> 校验代理类工具"
run_step mcp-proxy --version >/dev/null

echo "==> 检查远程 MCP 端点可达性"
run_step curl -sI https://mcp.deepwiki.com/mcp >/dev/null
run_step curl -sI https://learn.microsoft.com/api/mcp >/dev/null

echo "所有 MCP 服务已预热完成，可启动 Codex CLI。"
