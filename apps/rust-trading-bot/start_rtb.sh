#!/bin/bash
# RTB (Rust Trading Bot) 启动脚本
# 启动 integrated_ai_trader 包含 Web Server 和 Telegram 监听

set -e

cd "$(dirname "$0")"

echo "========================================"
echo "🚀 RTB Telegram信号系统启动中..."
echo "========================================"
echo ""

# 检查环境变量
if [ ! -f "/home/hanins/code/web3/.env" ]; then
    echo "❌ 错误: 根目录 .env 文件不存在"
    echo "路径: /home/hanins/code/web3/.env"
    exit 1
fi

echo "✅ 环境变量配置文件存在"

# 检查二进制文件
if [ ! -f "./target/release/integrated_ai_trader" ]; then
    echo "❌ 错误: integrated_ai_trader 未编译"
    echo "请运行: cargo build --bin integrated_ai_trader --release"
    exit 1
fi

echo "✅ integrated_ai_trader 二进制文件存在"

# 检查数据库目录
if [ ! -d "./data" ]; then
    echo "📁 创建 data 目录..."
    mkdir -p ./data
fi

echo "✅ 数据库目录准备完成"
echo ""

echo "========================================"
echo "📊 启动服务..."
echo "========================================"
echo "• Web API: http://localhost:8080"
echo "• 健康检查: http://localhost:8080/health"
echo "• Telegram信号: http://localhost:8080/api/telegram-signals"
echo ""
echo "前端启动命令 (新终端):"
echo "  cd web && npm run dev"
echo "  访问: http://localhost:5173/telegram-signals"
echo ""
echo "========================================"
echo ""

# 启动主程序
exec ./target/release/integrated_ai_trader
