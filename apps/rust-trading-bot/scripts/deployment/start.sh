#!/bin/bash

# AI交易机器人启动脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 启动AI交易机器人系统..."
echo ""

# 检查编译
if [ ! -f "target/release/integrated_ai_trader" ]; then
    echo "⚠️  未找到编译后的程序，开始编译..."
    cargo build --release --bin integrated_ai_trader
fi

# 停止旧进程
echo "🔄 检查并停止旧进程..."
OLD_PID=$(ps aux | grep "[i]ntegrated_ai_trader" | awk '{print $2}')
if [ -n "$OLD_PID" ]; then
    echo "  停止旧进程: $OLD_PID"
    kill $OLD_PID
    sleep 2
fi

# 启动交易机器人
echo "🤖 启动交易机器人..."
LOG_FILE="logs/trader_$(date +%Y%m%d_%H%M%S).log"
mkdir -p logs
nohup ./target/release/integrated_ai_trader > "$LOG_FILE" 2>&1 &
TRADER_PID=$!
echo "  PID: $TRADER_PID"
echo "  日志: $LOG_FILE"

# 等待启动
sleep 3

# 检查Web API
echo ""
echo "🌐 检查Web API..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "  ✅ Web API正常 (http://localhost:8080)"
else
    echo "  ❌ Web API未响应"
fi

# 检查前端
echo ""
echo "💻 检查前端服务..."
VITE_PID=$(ps aux | grep "node.*vite" | grep -v grep | awk '{print $2}' | head -1)
if [ -n "$VITE_PID" ]; then
    echo "  ✅ 前端运行中 (PID: $VITE_PID)"
    echo "  访问: http://localhost:5173"
else
    echo "  ⚠️  前端未运行，启动中..."
    cd web
    npm run dev > ../logs/vite.log 2>&1 &
    echo "  ✅ 前端已启动"
    echo "  访问: http://localhost:5173"
    cd ..
fi

echo ""
echo "✨ 系统启动完成！"
echo ""
echo "📊 服务地址:"
echo "  - Web监控面板: http://localhost:5173"
echo "  - API接口: http://localhost:8080/api/"
echo "  - 健康检查: http://localhost:8080/health"
echo ""
echo "📝 日志文件:"
echo "  - 交易机器人: $LOG_FILE"
echo "  - 前端服务: logs/vite.log"
echo ""
echo "🔍 监控命令:"
echo "  - 查看日志: tail -f $LOG_FILE"
echo "  - 系统诊断: ./system_check.sh"
echo "  - API测试: ./test_api.sh"
echo ""
