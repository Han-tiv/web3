#!/bin/bash
# DeepSeek Trading Bot 启动脚本

set -e

PROJECT_ROOT="/home/hanins/code/web3/apps/rust-trading-bot"
cd "$PROJECT_ROOT"

echo "🤖 DeepSeek AI Trading Bot 启动脚本"
echo "═══════════════════════════════════════════"
echo ""

# 检查环境变量
if [ ! -f "../.env" ]; then
    echo "❌ 错误: 找不到 .env 文件"
    echo "   请在 /home/hanins/code/web3/.env 中配置环境变量"
    exit 1
fi

echo "✅ 环境变量文件: ../env"

# 检查必要的环境变量
source ../.env

if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ 错误: DEEPSEEK_API_KEY 未设置"
    exit 1
fi

if [ -z "$BINANCE_API_KEY" ] && [ -z "$OKX_API_KEY" ]; then
    echo "❌ 错误: 未设置 BINANCE_API_KEY 或 OKX_API_KEY"
    exit 1
fi

echo "✅ 环境变量检查通过"
echo ""

# 编译检查
echo "🔨 检查编译状态..."
if ! cargo check --bin deepseek_trader --quiet 2>/dev/null; then
    echo "⚠️  需要重新编译..."
    cargo build --release --bin deepseek_trader
else
    echo "✅ 编译检查通过"
fi
echo ""

# 检查二进制文件
if [ ! -f "target/release/deepseek_trader" ]; then
    echo "🔨 编译 release 版本..."
    cargo build --release --bin deepseek_trader
    echo "✅ 编译完成"
fi
echo ""

# 显示二进制文件信息
echo "📦 程序信息:"
echo "   位置: $(pwd)/target/release/deepseek_trader"
echo "   大小: $(ls -lh target/release/deepseek_trader | awk '{print $5}')"
echo "   修改时间: $(stat -c %y target/release/deepseek_trader | cut -d. -f1)"
echo ""

# 询问是否继续
echo "⚠️  警告:"
echo "   - 这是真实交易机器人"
echo "   - 请确保已充分测试"
echo "   - 建议从小额开始"
echo ""
read -p "确认启动? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ 取消启动"
    exit 0
fi

echo ""
echo "🚀 启动 DeepSeek Trading Bot..."
echo "═══════════════════════════════════════════"
echo ""

# 设置日志级别
export RUST_LOG="${RUST_LOG:-info}"

# 运行
exec ./target/release/deepseek_trader
