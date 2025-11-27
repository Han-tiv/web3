#!/bin/bash

# 交易系统启动脚本 - 确保完整日志输出
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

# 加载环境变量（优先根目录 .env，其次本地 .env）
ENV_FILE="$PROJECT_ROOT/.env"
if [ -f "$ENV_FILE" ]; then
    # shellcheck disable=SC1090
    source "$ENV_FILE"
elif [ -f "$SCRIPT_DIR/.env" ]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/.env"
else
    echo "❌ 未找到环境变量文件: $ENV_FILE"
    echo "   请创建 .env 并包含交易所 API/Telegram 配置"
    exit 1
fi

# 设置日志级别
export RUST_LOG=info

# 创建日志文件
LOG_FILE="integrated_ai_trader.log"
echo "========================================" >> $LOG_FILE
echo "启动时间: $(date '+%Y-%m-%d %H:%M:%S')" >> $LOG_FILE
echo "进程ID: $$" >> $LOG_FILE
echo "日志级别: $RUST_LOG" >> $LOG_FILE
echo "========================================" >> $LOG_FILE
echo "" >> $LOG_FILE

# 启动交易系统
echo "🚀 启动 Integrated AI Trader..."
echo "📋 日志文件: $LOG_FILE"

nohup ./target/release/integrated_ai_trader >> $LOG_FILE 2>&1 &
TRADER_PID=$!

echo "✅ 交易系统已启动"
echo "📊 进程ID: $TRADER_PID"
echo "📂 日志路径: $(pwd)/$LOG_FILE"
echo ""
echo "查看实时日志: tail -f $LOG_FILE"
echo "停止系统: kill $TRADER_PID"

# 等待3秒检查进程是否正常启动
sleep 3

if ps -p "$TRADER_PID" > /dev/null; then
    echo "✅ 进程运行正常"
    tail -20 "$LOG_FILE"
else
    echo "❌ 进程启动失败，查看日志:"
    tail -50 "$LOG_FILE"
fi
