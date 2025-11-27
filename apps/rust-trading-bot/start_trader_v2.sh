#!/bin/bash
# Valuescan V2 启动脚本
# 使用方式: bash start_trader_v2.sh [v1|v2]

set -e

cd "$(dirname "$0")"

# 默认使用V2
VERSION="${1:-v2}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🚀 Rust AI交易机器人启动 - Valuescan $VERSION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 设置环境变量
if [ "$VERSION" = "v2" ]; then
    export USE_VALUESCAN_V2=true
    echo "✅ 使用 Valuescan V2 (关键位50% + 资金流30% + 技术指标20%)"
    echo "   - 评分系统: 0-10分, ≥6分才开仓"
    echo "   - 开仓检查: 10项检查, 8项满足才开"
    echo "   - 持仓优先级: 关键位(60%) > K线反转(30%) > 盈利时间(10%)"
    echo ""
elif [ "$VERSION" = "v1" ]; then
    export USE_VALUESCAN_V2=false
    echo "✅ 使用 Valuescan V1 (K线形态60% + 资金流30% + 技术指标10%)"
    echo ""
else
    echo "❌ 错误: 无效的版本参数 '$VERSION'"
    echo "   使用方式: bash start_trader_v2.sh [v1|v2]"
    exit 1
fi

# 检查环境变量
echo "📋 环境变量检查:"
echo "   USE_VALUESCAN_V2 = $USE_VALUESCAN_V2"
echo ""

# 停止旧进程
echo "🛑 停止旧进程..."
pkill -f "integrated_ai_trader" 2>/dev/null || true
sleep 2
echo ""

# 编译程序
echo "🔨 编译程序..."
cargo build --bin integrated_ai_trader --release
if [ $? -ne 0 ]; then
    echo "❌ 编译失败!"
    exit 1
fi
echo ""

# 启动程序
echo "🚀 启动交易机器人..."
echo "   日志文件: trader.log"
echo "   PID文件: trader.pid"
echo ""

nohup target/release/integrated_ai_trader > trader.log 2>&1 &
TRADER_PID=$!
echo $TRADER_PID > trader.pid

echo "✅ 交易机器人已启动!"
echo "   PID: $TRADER_PID"
echo "   版本: Valuescan $VERSION"
echo ""

# 等待启动
echo "⏳ 等待启动(5秒)..."
sleep 5
echo ""

# 检查进程状态
if ps -p $TRADER_PID > /dev/null 2>&1; then
    echo "✅ 进程运行正常"
    echo ""

    # 显示最新日志
    echo "📊 最新日志 (最近20行):"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    tail -n 20 trader.log
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    echo "💡 查看实时日志:"
    echo "   tail -f trader.log"
    echo ""
    echo "💡 检查Valuescan版本:"
    echo "   grep 'Valuescan版本' trader.log"
    echo ""
    echo "💡 查看V2评分信息:"
    echo "   grep 'V2评分' trader.log"
    echo ""
    echo "💡 停止程序:"
    echo "   kill $TRADER_PID"
    echo "   或 bash stop_trader.sh"
    echo ""
else
    echo "❌ 进程启动失败! 查看日志:"
    echo ""
    tail -n 50 trader.log
    exit 1
fi

echo "🎉 启动完成!"
