#!/bin/bash

# AI交易机器人停止脚本

echo "🛑 停止AI交易机器人系统..."
echo ""

# 停止交易机器人
TRADER_PID=$(ps aux | grep "[i]ntegrated_ai_trader" | awk '{print $2}')
if [ -n "$TRADER_PID" ]; then
    echo "🤖 停止交易机器人 (PID: $TRADER_PID)"
    kill $TRADER_PID
    sleep 2
    if ps -p $TRADER_PID > /dev/null 2>&1; then
        echo "  ⚠️  进程未响应，强制停止"
        kill -9 $TRADER_PID
    fi
    echo "  ✅ 交易机器人已停止"
else
    echo "  ℹ️  交易机器人未运行"
fi

# 停止前端服务
VITE_PIDS=$(ps aux | grep "node.*vite" | grep -v grep | awk '{print $2}')
if [ -n "$VITE_PIDS" ]; then
    echo ""
    echo "💻 停止前端服务"
    for PID in $VITE_PIDS; do
        echo "  停止 PID: $PID"
        kill $PID 2>/dev/null
    done
    sleep 1
    echo "  ✅ 前端服务已停止"
else
    echo "  ℹ️  前端服务未运行"
fi

# 停止shell进程
SH_PIDS=$(ps aux | grep "sh -c vite" | grep -v grep | awk '{print $2}')
if [ -n "$SH_PIDS" ]; then
    for PID in $SH_PIDS; do
        kill $PID 2>/dev/null
    done
fi

echo ""
echo "✅ 系统已停止"
