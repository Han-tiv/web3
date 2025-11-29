#!/bin/bash

# Gemini ETH-USDT 分析器启动脚本

echo "🚀 启动 Gemini ETH-USDT 分析器..."

# 加载环境变量
if [ -f "/home/hanins/code/web3/.env" ]; then
    export $(grep -v '^#' /home/hanins/code/web3/.env | xargs)
    echo "✅ 已加载环境变量"
else
    echo "❌ 未找到 .env 文件"
    exit 1
fi

# 检查是否已在运行
if pgrep -f "gemini_eth_analyzer" > /dev/null; then
    echo "⚠️  分析器已在运行"
    echo "   PID: $(pgrep -f gemini_eth_analyzer)"
    exit 1
fi

# 启动分析器
cd /home/hanins/code/web3/apps/rust-trading-bot

# 后台运行
nohup ./target/release/gemini_eth_analyzer > gemini_eth.log 2>&1 &
PID=$!

sleep 2

# 检查是否启动成功
if ps -p $PID > /dev/null; then
    echo "✅ 分析器已启动"
    echo "📊 进程ID: $PID"
    echo "📂 日志文件: gemini_eth.log"
    echo ""
    echo "查看实时日志: tail -f gemini_eth.log"
    echo "停止分析器: kill $PID"
else
    echo "❌ 启动失败，查看日志:"
    tail -50 gemini_eth.log
    exit 1
fi
