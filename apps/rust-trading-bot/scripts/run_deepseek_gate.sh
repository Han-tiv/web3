#!/bin/bash
# DeepSeek AI Trading Bot - Gate.io 版本
# 使用 Gate.io 交易所进行 AI 自动交易

set -e

PROJECT_ROOT="/home/hanins/code/web3/apps/rust-trading-bot"
cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════"
echo "🤖 DeepSeek AI Trading Bot - Gate.io"
echo "═══════════════════════════════════════════"
echo ""

# 检查环境变量
if [ ! -f "../../.env" ]; then
    echo "❌ 错误: 找不到 .env 文件"
    echo "   请在 /home/hanins/code/web3/.env 中配置环境变量"
    exit 1
fi

echo "✅ 环境变量文件: ../../.env"

# 加载环境变量检查
source ../../.env

if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ 错误: DEEPSEEK_API_KEY 未设置"
    echo "   请在 .env 中添加: DEEPSEEK_API_KEY=your_key"
    exit 1
fi

if [ -z "$GATE_API_KEY" ] || [ -z "$GATE_SECRET" ]; then
    echo "❌ 错误: Gate.io API 密钥未设置"
    echo "   请在 .env 中添加:"
    echo "   GATE_API_KEY=your_key"
    echo "   GATE_SECRET=your_secret"
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

# 显示程序信息
echo "📦 程序信息:"
echo "   位置: $(pwd)/target/release/deepseek_trader"
echo "   大小: $(ls -lh target/release/deepseek_trader 2>/dev/null | awk '{print $5}')"
echo ""

# 显示配置信息
echo "⚙️  交易配置:"
echo "   交易所: Gate.io"
echo "   交易对: BTC/USDT"
echo "   杠杆: 10x"
echo "   周期: 15分钟"
echo ""

# 警告信息
echo "⚠️  重要提示:"
echo "   - 这是实盘交易，会使用真实资金"
echo "   - 建议先用小额测试"
echo "   - 随时可以按 Ctrl+C 停止"
echo "   - 请确保已充分了解风险"
echo ""

read -p "确认启动 Gate.io AI 交易? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ 取消启动"
    exit 0
fi

echo ""
echo "🚀 启动 DeepSeek AI Trading Bot (Gate.io)..."
echo "═══════════════════════════════════════════"
echo ""

# 设置日志级别
export RUST_LOG="${RUST_LOG:-info}"

# 运行
exec ./target/release/deepseek_trader
