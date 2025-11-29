#!/bin/bash
#
# 混合架构集成测试
# 测试Python监控 → Rust引擎的完整信号流

set -e

echo "🧪 开始集成测试: Python监控 → Rust交易引擎"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. 检查Rust引擎是否运行
echo ""
echo "📡 第1步: 检查Rust交易引擎状态"
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ Rust引擎在线 (http://localhost:8080)"
else
    echo "❌ Rust引擎未运行,请先启动:"
    echo "   cd /home/hanins/code/web3/apps/rust-trading-bot"
    echo "   bash start_trader.sh"
    exit 1
fi

# 2. 测试信号发送
echo ""
echo "📨 第2步: 模拟Python发送交易信号"
RESPONSE=$(curl -s -X POST http://localhost:8080/api/signals \
    -H "Content-Type: application/json" \
    -d '{
        "symbol": "BTCUSDT",
        "raw_message": "测试信号: BTCUSDT LONG 95000 SL:94000 TP:96000 10X",
        "timestamp": 1700000000.0
    }')

echo "📝 Rust响应: $RESPONSE"

# 检查响应状态
if echo "$RESPONSE" | grep -q '"status":"received"'; then
    echo "✅ 信号成功接收"
else
    echo "❌ 信号接收失败"
    echo "响应内容: $RESPONSE"
    exit 1
fi

# 3. 验证数据库记录
echo ""
echo "🗄️  第3步: 验证数据库保存"
SIGNALS=$(curl -s http://localhost:8080/api/telegram-signals)
if echo "$SIGNALS" | grep -q "BTCUSDT"; then
    echo "✅ 信号已保存到数据库"
    echo "$SIGNALS" | jq '.[0]' 2>/dev/null || echo "$SIGNALS"
else
    echo "❌ 数据库未找到信号记录"
    exit 1
fi

# 4. 测试完整流程
echo ""
echo "🔄 第4步: 测试多个信号"
for SYMBOL in "ETHUSDT" "SOLUSDT" "BNBUSDT"; do
    curl -s -X POST http://localhost:8080/api/signals \
        -H "Content-Type: application/json" \
        -d "{
            \"symbol\": \"$SYMBOL\",
            \"raw_message\": \"测试信号: $SYMBOL SHORT\",
            \"timestamp\": $(date +%s).0
        }" > /dev/null
    echo "✅ 已发送: $SYMBOL"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 集成测试完成!"
echo ""
echo "📊 当前系统状态:"
curl -s http://localhost:8080/api/status | jq '.' 2>/dev/null || curl -s http://localhost:8080/api/status
echo ""
echo "🎯 下一步:"
echo "   1. 启动Python监控: bash start_monitor.sh"
echo "   2. 查看实时日志: tail -f telegram_monitor.log"
echo "   3. 打开前端面板: http://localhost:5173"
