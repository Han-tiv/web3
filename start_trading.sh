#!/bin/bash
#
# 混合架构交易系统 - 一键启动脚本
#

set -e

PROJECT_ROOT="/home/hanins/code/web3"
RUST_DIR="$PROJECT_ROOT/apps/rust-trading-bot"
PYTHON_DIR="$PROJECT_ROOT/apps/python-telegram-monitor"
TELEGRAM_WORKER="${TELEGRAM_WORKER:-signal_forwarder}"

case "$TELEGRAM_WORKER" in
    signal_forwarder)
        PYTHON_ENTRY="signal_forwarder.py"
        PYTHON_DESC="Signal Forwarder"
        PYTHON_LOG="telegram_forwarder.log"
        PYTHON_PID_NAME="telegram.pid"
        ;;
    telegram_monitor|*)
        PYTHON_ENTRY="telegram_monitor.py"
        PYTHON_DESC="Telegram Monitor"
        PYTHON_LOG="telegram_monitor.log"
        PYTHON_PID_NAME="monitor.pid"
        TELEGRAM_WORKER="telegram_monitor"
        ;;
esac
PYTHON_PID_FILE="$PYTHON_DIR/$PYTHON_PID_NAME"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 混合架构交易系统启动"
echo "   Python ($PYTHON_DESC) + Rust (交易引擎)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 检查环境配置
echo "🔍 第1步: 检查环境配置"
if [ ! -f "$PROJECT_ROOT/.env" ]; then
    echo "❌ 错误: 未找到 $PROJECT_ROOT/.env"
    echo "   请先配置环境变量"
    exit 1
fi
echo "✅ 环境配置存在"

# 检查Python依赖
echo ""
echo "🔍 第2步: 检查Python依赖"
if [ -d "$PYTHON_DIR/venv" ]; then
    echo "✅ 虚拟环境已存在"
    # 激活venv并检查依赖
    source "$PYTHON_DIR/venv/bin/activate"
    PYTHON_IMPORTS=("telethon" "dotenv" "aiohttp" "colorlog")
    if [ "$TELEGRAM_WORKER" = "signal_forwarder" ]; then
        PYTHON_IMPORTS+=("httpx")
    fi
    for module in "${PYTHON_IMPORTS[@]}"; do
        if ! python3 -c "import ${module}" 2>/dev/null; then
            echo "⚠️  虚拟环境缺少 ${module}, 请运行:"
            echo "   cd $PYTHON_DIR && source venv/bin/activate && pip install -r requirements.txt"
            exit 1
        fi
    done
    echo "✅ Python依赖已安装"
else
    echo "⚠️  虚拟环境不存在"
    echo "   正在创建虚拟环境..."
    cd "$PYTHON_DIR"
    python3 -m venv venv
    source venv/bin/activate
    echo "   正在安装依赖..."
    pip install -r requirements.txt
    echo "✅ 虚拟环境创建完成"
    cd "$PROJECT_ROOT"
fi

# 检查Rust编译
echo ""
echo "🔍 第3步: 检查Rust编译状态"
if [ ! -f "$RUST_DIR/target/release/integrated_ai_trader" ]; then
    echo "⚠️  Rust未编译,正在编译(可能需要几分钟)..."
    cd "$RUST_DIR"
    cargo build --release
else
    echo "✅ Rust已编译"
fi

# 停止已有进程
echo ""
echo "🛑 第4步: 停止已有进程"
pkill -f "integrated_ai_trader" 2>/dev/null && echo "   - 已停止旧的Rust引擎" || echo "   - 无需停止Rust引擎"
pkill -f "signal_forwarder.py" 2>/dev/null && echo "   - 已停止旧的Signal Forwarder" || echo "   - 无需停止Signal Forwarder"
pkill -f "telegram_monitor.py" 2>/dev/null && echo "   - 已停止旧的Telegram Monitor" || echo "   - 无需停止Telegram Monitor"
pkill -f "monitor_and_restart.sh" 2>/dev/null && echo "   - 已停止旧的健康监控" || echo "   - 无需停止健康监控"

# 启动Rust交易引擎
echo ""
echo "🚀 第5步: 启动Rust交易引擎"
cd "$RUST_DIR"
nohup cargo run --release --bin integrated_ai_trader > trader.log 2>&1 &
RUST_PID=$!
echo $RUST_PID > trader.pid
echo "   PID: $RUST_PID"

# 等待Rust引擎启动
echo "   等待引擎启动..."
for i in {1..10}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "   ✅ Rust引擎已启动 (http://localhost:8080)"
        break
    fi
    sleep 1
done

if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "   ❌ Rust引擎启动失败,请检查日志:"
    echo "      tail -f $RUST_DIR/trader.log"
    exit 1
fi

# 启动Python监控
echo ""
echo "🚀 第6步: 启动Python $PYTHON_DESC"
cd "$PYTHON_DIR"
# 使用venv中的Python
source venv/bin/activate
nohup python3 "$PYTHON_ENTRY" > "$PYTHON_LOG" 2>&1 &
PYTHON_PID=$!
echo $PYTHON_PID > "$PYTHON_PID_FILE"
echo $PYTHON_PID > "$PROJECT_ROOT/monitor.pid"
echo "   PID: $PYTHON_PID"
echo "   日志: $PYTHON_DIR/$PYTHON_LOG"

# 等待Python监控启动
echo "   等待监控启动..."
sleep 3
if ps -p $PYTHON_PID > /dev/null; then
    echo "   ✅ Python $PYTHON_DESC 已启动"
else
    echo "   ❌ Python $PYTHON_DESC 启动失败,请检查日志:"
    echo "      tail -f $PYTHON_DIR/$PYTHON_LOG"
    exit 1
fi

# 启动Telegram健康监控
echo ""
echo "🔍 第7步: 启动Telegram健康监控"
if bash "$RUST_DIR/start_monitor.sh"; then
    echo "   ✅ 连接监控已运行"
else
    echo "   ⚠️  连接监控启动失败，请查看 $RUST_DIR/monitor.log"
fi

# 显示状态
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 系统启动完成!"
echo ""
echo "📊 系统状态:"
curl -s http://localhost:8080/api/status | python3 -m json.tool 2>/dev/null || curl -s http://localhost:8080/api/status
echo ""
echo "📂 进程信息:"
echo "   - Rust引擎 PID: $RUST_PID"
echo "   - Python监控 PID: $PYTHON_PID"
if [ -f "$RUST_DIR/monitor_and_restart.pid" ]; then
    echo "   - 健康监控 PID: $(cat "$RUST_DIR/monitor_and_restart.pid")"
fi
echo ""
echo "📁 日志位置:"
echo "   - Rust引擎: $RUST_DIR/trader.log"
echo "   - Python监控: $PYTHON_DIR/$PYTHON_LOG"
echo "   - Telegram健康监控: $RUST_DIR/monitor.log"
echo ""
echo "🌐 Web界面:"
echo "   - API服务: http://localhost:8080"
echo "   - 前端面板: http://localhost:5173 (需手动启动)"
echo ""
echo "🔍 实时监控:"
echo "   tail -f $RUST_DIR/trader.log $PYTHON_DIR/$PYTHON_LOG"
echo ""
echo "🛑 停止系统:"
echo "   bash $PROJECT_ROOT/stop_trading.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
