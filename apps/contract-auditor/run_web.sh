#!/usr/bin/env bash

# 启动 Streamlit Web 界面

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 若存在本地虚拟环境则自动激活
if [ -f ".venv/bin/activate" ]; then
  # shellcheck disable=SC1091
  source ".venv/bin/activate"
fi

exec streamlit run src/web_ui/app.py "$@"

