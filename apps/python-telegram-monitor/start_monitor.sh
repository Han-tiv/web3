#!/bin/bash
#
# Telegram监控启动脚本 (支持venv)
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

WORKER="${1:-${TELEGRAM_WORKER:-signal_forwarder}}"

case "$WORKER" in
    signal_forwarder)
        ENTRY_FILE="signal_forwarder.py"
        DESC="Signal Forwarder"
        EXTRA_IMPORTS=("httpx")
        ;;
    telegram_monitor|*)
        ENTRY_FILE="telegram_monitor.py"
        DESC="Telegram Monitor"
        WORKER="telegram_monitor"
        EXTRA_IMPORTS=()
        ;;
esac

echo "🚀 启动$DESC..."

# 检查并激活虚拟环境
if [ -d "venv" ]; then
    echo "✅ 使用虚拟环境 venv/"
    source venv/bin/activate
else
    echo "⚠️  未找到虚拟环境,使用系统Python"
fi

# 检查依赖
PY_IMPORTS=("telethon" "dotenv" "aiohttp" "colorlog")
PY_IMPORTS+=("${EXTRA_IMPORTS[@]}")
for module in "${PY_IMPORTS[@]}"; do
    if ! python3 -c "import ${module}" 2>/dev/null; then
        echo "❌ 缺少依赖 ${module},请先运行:"
        echo "   python3 -m venv venv"
        echo "   source venv/bin/activate"
        echo "   pip install -r requirements.txt"
        exit 1
    fi
done

# 检查配置
python3 -c "from config import validate_config; validate_config()" || {
    echo "❌ 配置验证失败,请检查 .env 文件"
    echo ""
    echo "需要配置以下变量:"
    echo "  TELEGRAM_API_ID=2040"
    echo "  TELEGRAM_API_HASH=your_hash"
    echo "  TELEGRAM_PHONE=+17578852234"
    echo "  TELEGRAM_CHANNELS=-1001234567890,@channel_name"
    exit 1
}

# 启动监控
echo "✅ 配置验证通过"
echo "📡 正在连接Telegram..."

python3 "$ENTRY_FILE"
