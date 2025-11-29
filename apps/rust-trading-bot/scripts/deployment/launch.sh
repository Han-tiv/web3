#!/bin/bash
# 完整启动流程 - 包含P0-P1修复的Rust AI交易机器人
# 生成时间: 2025-11-24

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=========================================="
echo "🚀 Rust AI交易机器人 - 完整启动流程"
echo "=========================================="
echo ""

# 1. 环境检查
echo "📋 [1/5] 环境检查..."
cd "$SCRIPT_DIR"

# 检查 .env 文件
ENV_FILE="$PROJECT_ROOT/.env"
if [ ! -f "$ENV_FILE" ]; then
    echo "❌ 未找到环境变量文件: $ENV_FILE"
    echo "   请创建包含以下配置的 .env 文件:"
    echo "   - BINANCE_API_KEY"
    echo "   - BINANCE_SECRET"
    echo "   - GEMINI_API_KEY"
    echo "   - TELEGRAM_API_ID"
    echo "   - TELEGRAM_API_HASH"
    exit 1
fi

# 加载环境变量
source "$ENV_FILE"
echo "✅ 环境变量已加载: $ENV_FILE"

# 检查必需的环境变量
REQUIRED_VARS=("BINANCE_API_KEY" "BINANCE_SECRET" "GEMINI_API_KEY")
for VAR in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!VAR}" ]; then
        echo "❌ 缺少必需的环境变量: $VAR"
        exit 1
    fi
done
echo "✅ 必需环境变量已配置"
echo ""

# 2. 编译检查
echo "🔨 [2/5] 编译检查..."
cargo check --bin integrated_ai_trader 2>&1 | tail -5
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "❌ 编译检查失败"
    exit 1
fi
echo "✅ 编译检查通过"
echo ""

# 3. 停止旧进程
echo "🛑 [3/5] 停止旧进程..."
OLD_PID=$(pgrep -f "integrated_ai_trader" || true)
if [ -n "$OLD_PID" ]; then
    echo "   发现旧进程 PID: $OLD_PID"
    kill $OLD_PID && sleep 2
    echo "✅ 旧进程已停止"
else
    echo "✅ 无旧进程运行"
fi
echo ""

# 4. Release编译
echo "⚙️  [4/5] Release编译 (可能需要1-2分钟)..."
cargo build --release --bin integrated_ai_trader 2>&1 | grep -E "(Compiling|Finished|error)" || true
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "❌ Release编译失败"
    exit 1
fi
echo "✅ Release编译完成"
echo ""

# 5. 启动程序
echo "🚀 [5/5] 启动交易机器人..."

# 设置日志
export RUST_LOG=info
export RUST_BACKTRACE=1
LOG_FILE="trader_$(date +%Y%m%d_%H%M%S).log"

echo "========================================" >> $LOG_FILE
echo "启动时间: $(date '+%Y-%m-%d %H:%M:%S')" >> $LOG_FILE
echo "修复版本: P0-P1 (2025-11-24)" >> $LOG_FILE
echo "  - P0.1: 部分平仓最小金额检查" >> $LOG_FILE
echo "  - P0.2: 15%强制全仓止盈" >> $LOG_FILE
echo "  - P1.1: 持仓检查间隔 600s→180s" >> $LOG_FILE
echo "  - P1.2: 30分钟/-3%快速止损" >> $LOG_FILE
echo "  - P1.3: Valuescan V2阈值≥6.5" >> $LOG_FILE
echo "========================================" >> $LOG_FILE
echo "" >> $LOG_FILE

# 后台启动
nohup ./target/release/integrated_ai_trader >> $LOG_FILE 2>&1 &
TRADER_PID=$!

echo "✅ 交易机器人已启动"
echo ""
echo "=========================================="
echo "📊 运行状态"
echo "=========================================="
echo "进程ID: $TRADER_PID"
echo "日志文件: $(pwd)/$LOG_FILE"
echo "Web API: http://localhost:8080"
echo ""
echo "常用命令:"
echo "  查看实时日志: tail -f $LOG_FILE"
echo "  查看进程: ps aux | grep integrated_ai_trader"
echo "  停止程序: kill $TRADER_PID"
echo "  查看P1.2止损: grep '快速止损触发' $LOG_FILE"
echo "  查看P1.3过滤: grep 'Valuescan V2评分.*不足6.5' $LOG_FILE"
echo ""

# 等待启动
echo "⏳ 等待启动 (5秒)..."
sleep 5

# 检查进程状态
if ps -p "$TRADER_PID" > /dev/null; then
    echo "✅ 进程运行正常"
    echo ""
    echo "=========================================="
    echo "📋 最新日志 (最近20行)"
    echo "=========================================="
    tail -20 "$LOG_FILE"
    echo ""
    echo "✅ 启动完成! 使用 'tail -f $LOG_FILE' 查看实时日志"
else
    echo "❌ 进程启动失败，查看完整日志:"
    echo ""
    tail -50 "$LOG_FILE"
    exit 1
fi
