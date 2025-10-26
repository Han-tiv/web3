#!/bin/bash
# 交易系统启动脚本（仅启动 signal_trader）
# 已停用 profit_monitor，聚焦信号交易守护进程

set -e

# 统一设置时区
export TZ=Asia/Shanghai

echo "🚀 启动 signal_trader 交易系统"
echo "═══════════════════════════════════════════════"

# 校验并加载环境变量
if [ ! -f "../../.env" ]; then
    echo "❌ 未找到根目录 .env 文件"
    echo "请确保 /home/hanins/code/.env 存在并配置正确"
    exit 1
fi

echo "✅ 加载环境变量..."
set -a
source ../../.env
set +a

# 核心环境变量校验
required_vars=(
  "BINANCE_API_KEY" "BINANCE_SECRET_KEY"
  "TELEGRAM_API_ID" "TELEGRAM_API_HASH" "TARGET_CHANNEL_ID"
  "SIGNAL_LEVERAGE" "SIGNAL_MARGIN" "SIGNAL_MARGIN_TYPE"
  "SIGNAL_MULTI_ASSET_MODE" "SIGNAL_STOP_LOSS_PERCENT"
)

for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ 缺少环境变量: $var"
        exit 1
    fi
done

echo "✅ 环境变量验证通过"
echo ""

# 展示当前配置，便于人工核对
echo "📊 当前配置:"
echo "   杠杆倍数: ${SIGNAL_LEVERAGE}x"
echo "   保证金: ${SIGNAL_MARGIN} USDT"
echo "   仓位模式: ${SIGNAL_MARGIN_TYPE}"
echo "   保证金资产模式: ${SIGNAL_MULTI_ASSET_MODE}"
echo "   止损阈值: ${SIGNAL_STOP_LOSS_PERCENT}%"
echo "   交易状态: ${SIGNAL_TRADING_ENABLED}"
echo "   Binance 环境: $(if [ "$BINANCE_TESTNET" = "true" ]; then echo "测试网"; else echo "主网"; fi)"
echo ""

# 若已有实例运行则先清理，避免重复持仓
if pgrep -f "signal_trader" > /dev/null 2>&1; then
    echo "⚠️  已检测到 signal_trader 正在运行，准备停止..."
    pkill -f "signal_trader" || true
    sleep 2
fi

# 清理旧状态文件
echo "🧹 清理交易锁文件..."
rm -rf ./trading_locks ./status 2>/dev/null || true

echo "🚀 启动程序..."
nohup ./target/release/signal_trader > signal_trader.log 2>&1 &
SIGNAL_PID=$!
echo "   signal_trader 进程 ID: $SIGNAL_PID"

# 等待数秒检查启动是否成功
sleep 5

if ! kill -0 "$SIGNAL_PID" 2>/dev/null; then
    echo "❌ signal_trader 启动失败"
    echo "请查看日志: tail signal_trader.log"
    exit 1
fi

echo ""
echo "✅ signal_trader 已后台运行"
echo "═══════════════════════════════════════════════"
echo "📝 日志文件: signal_trader.log"
echo "🔧 管理命令:"
echo "   查看日志: tail -f signal_trader.log"
echo "   停止进程: pkill -f signal_trader"
echo "   查看进程: ps aux | grep signal_trader"
echo ""
echo "💡 建议改用 supervisor/systemd 守护进程以获得更可靠的重启与监控能力。"
