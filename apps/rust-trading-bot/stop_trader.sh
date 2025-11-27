#!/bin/bash
# 停止交易机器人脚本

cd "$(dirname "$0")"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🛑 停止 Rust AI交易机器人"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 从PID文件读取
if [ -f trader.pid ]; then
    PID=$(cat trader.pid)
    if ps -p $PID > /dev/null 2>&1; then
        echo "🛑 停止进程 PID: $PID"
        kill $PID
        sleep 2

        # 确认是否停止
        if ps -p $PID > /dev/null 2>&1; then
            echo "⚠️  进程未响应,强制停止..."
            kill -9 $PID
            sleep 1
        fi

        echo "✅ 进程已停止"
        rm -f trader.pid
    else
        echo "ℹ️  进程 $PID 不存在"
        rm -f trader.pid
    fi
else
    echo "ℹ️  未找到 trader.pid 文件"
fi

# 额外检查所有相关进程
echo ""
echo "🔍 检查残留进程..."
pkill -f "integrated_ai_trader" 2>/dev/null && echo "✅ 已清理残留进程" || echo "✅ 无残留进程"

echo ""
echo "✅ 停止完成!"
