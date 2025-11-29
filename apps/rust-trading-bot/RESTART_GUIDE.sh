#!/bin/bash

# 交易系统重启脚本
# 生成时间: 2025-11-29

echo "═══════════════════════════════════════════════════════════"
echo "         🔄 交易系统重启脚本"
echo "═══════════════════════════════════════════════════════════"
echo ""

# 1. 停止旧程序
echo "步骤1: 停止旧程序..."
OLD_PID=$(ps aux | grep integrated_ai_trader | grep -v grep | awk '{print $2}')

if [ -n "$OLD_PID" ]; then
    echo "  找到进程 PID: $OLD_PID"
    kill $OLD_PID
    echo "  等待程序优雅退出..."
    sleep 3
    
    # 检查是否还在运行
    if ps -p $OLD_PID > /dev/null 2>&1; then
        echo "  ⚠️  程序未退出，强制停止..."
        kill -9 $OLD_PID
        sleep 1
    fi
    echo "  ✅ 旧程序已停止"
else
    echo "  ℹ️  未找到运行中的程序"
fi

echo ""

# 2. 重新编译
echo "步骤2: 重新编译（Release模式）..."
cargo build --release --bin integrated_ai_trader

if [ $? -ne 0 ]; then
    echo "  ❌ 编译失败！"
    exit 1
fi

echo "  ✅ 编译成功"
echo ""

# 3. 启动新程序
echo "步骤3: 启动新程序..."

# 确保日志目录存在
mkdir -p logs

# 后台启动
nohup ./target/release/integrated_ai_trader > logs/output.log 2>&1 &
NEW_PID=$!

echo "  ✅ 程序已启动"
echo "  新进程 PID: $NEW_PID"
echo ""

# 4. 验证启动
echo "步骤4: 验证启动状态..."
sleep 3

if ps -p $NEW_PID > /dev/null 2>&1; then
    echo "  ✅ 程序运行中"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "         ✅ 重启完成！"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "查看日志:"
    echo "  tail -f logs/startup.log"
    echo ""
    echo "查看输出:"
    echo "  tail -f logs/output.log"
    echo ""
    echo "检查进程:"
    echo "  ps aux | grep integrated_ai_trader"
    echo ""
    echo "测试API:"
    echo "  curl http://localhost:8080/api/status"
    echo ""
else
    echo "  ❌ 程序启动失败！"
    echo ""
    echo "查看错误日志:"
    echo "  tail -50 logs/output.log"
    exit 1
fi
