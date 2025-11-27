#!/bin/bash
#
# 混合架构交易系统 - 停止脚本
#

set -e

PROJECT_ROOT="/home/hanins/code/web3"
RUST_DIR="$PROJECT_ROOT/apps/rust-trading-bot"
PYTHON_DIR="$PROJECT_ROOT/apps/python-telegram-monitor"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🛑 停止混合架构交易系统"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 停止Rust引擎
if [ -f "$RUST_DIR/trader.pid" ]; then
    RUST_PID=$(cat "$RUST_DIR/trader.pid")
    if ps -p $RUST_PID > /dev/null 2>&1; then
        echo "🛑 停止Rust引擎 (PID: $RUST_PID)"
        kill $RUST_PID
        sleep 2
        if ps -p $RUST_PID > /dev/null 2>&1; then
            echo "   强制停止..."
            kill -9 $RUST_PID
        fi
        echo "✅ Rust引擎已停止"
    else
        echo "⚠️  Rust引擎未运行 (PID: $RUST_PID)"
    fi
    rm -f "$RUST_DIR/trader.pid"
else
    echo "⚠️  未找到Rust引擎PID文件"
    # 尝试按名称停止
    pkill -f "integrated_ai_trader" && echo "✅ 已停止Rust引擎" || echo "   Rust引擎未运行"
fi

# 停止Python监控
stop_python_worker() {
    local PID_FILE="$1"
    local DESC="$2"

    if [ ! -f "$PID_FILE" ]; then
        return 1
    fi

    local WORKER_PID
    WORKER_PID=$(cat "$PID_FILE")
    if ps -p "$WORKER_PID" > /dev/null 2>&1; then
        echo "🛑 停止$DESC (PID: $WORKER_PID)"
        kill "$WORKER_PID"
        sleep 2
        if ps -p "$WORKER_PID" > /dev/null 2>&1; then
            echo "   强制停止..."
            kill -9 "$WORKER_PID"
        fi
        echo "✅ $DESC 已停止"
    else
        echo "⚠️  $DESC 未运行 (PID: $WORKER_PID)"
    fi
    rm -f "$PID_FILE"
    return 0
}

if ! stop_python_worker "$PYTHON_DIR/telegram.pid" "Signal Forwarder"; then
    if ! stop_python_worker "$PYTHON_DIR/monitor.pid" "Telegram Monitor"; then
        echo "⚠️  未找到Python监控PID文件"
        pkill -f "signal_forwarder.py" 2>/dev/null && echo "✅ 已停止Signal Forwarder" || true
        pkill -f "telegram_monitor.py" 2>/dev/null && echo "✅ 已停止Telegram Monitor" || echo "   Python监控未运行"
    fi
fi

rm -f "$PROJECT_ROOT/monitor.pid" 2>/dev/null || true

# 停止Telegram健康监控
if [ -f "$RUST_DIR/monitor_and_restart.pid" ]; then
    MONITOR_PID=$(cat "$RUST_DIR/monitor_and_restart.pid")
    if ps -p "$MONITOR_PID" > /dev/null 2>&1; then
        echo "🛑 停止Telegram健康监控 (PID: $MONITOR_PID)"
        kill "$MONITOR_PID"
        sleep 2
        if ps -p "$MONITOR_PID" > /dev/null 2>&1; then
            echo "   强制停止..."
            kill -9 "$MONITOR_PID"
        fi
        echo "✅ 健康监控已停止"
    else
        echo "⚠️  健康监控未运行 (PID: $MONITOR_PID)"
    fi
    rm -f "$RUST_DIR/monitor_and_restart.pid"
else
    pkill -f "monitor_and_restart.sh" 2>/dev/null && echo "✅ 已停止健康监控" || echo "⚠️  健康监控未运行"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 系统已停止"
echo ""
echo "📁 日志保留在:"
echo "   - Rust引擎: $RUST_DIR/trader.log"
echo "   - Python监控: $PYTHON_DIR/telegram_forwarder.log 或 $PYTHON_DIR/telegram_monitor.log"
echo "   - 健康监控: $RUST_DIR/monitor.log"
echo ""
echo "🔄 重新启动:"
echo "   bash $PROJECT_ROOT/start_trading.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
