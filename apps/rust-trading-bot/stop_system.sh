#!/bin/bash
# 停止整个交易系统

echo "🛑 停止Valuescan V2交易系统..."
echo

# 停止Python监控器
if [ -f "/home/hanins/code/web3/apps/python-telegram-monitor/telegram.pid" ]; then
    PYTHON_PID=$(cat /home/hanins/code/web3/apps/python-telegram-monitor/telegram.pid)
    if ps -p $PYTHON_PID > /dev/null 2>&1; then
        kill $PYTHON_PID
        echo "✅ Python监控器已停止 (PID: $PYTHON_PID)"
    else
        echo "⚠️  Python监控器未运行"
    fi
    rm -f /home/hanins/code/web3/apps/python-telegram-monitor/telegram.pid
else
    echo "⚠️  未找到Python PID文件"
fi

# 停止Rust引擎
if [ -f "/home/hanins/code/web3/apps/rust-trading-bot/trader.pid" ]; then
    RUST_PID=$(cat /home/hanins/code/web3/apps/rust-trading-bot/trader.pid)
    if ps -p $RUST_PID > /dev/null 2>&1; then
        kill $RUST_PID
        echo "✅ Rust引擎已停止 (PID: $RUST_PID)"
    else
        echo "⚠️  Rust引擎未运行"
    fi
    rm -f /home/hanins/code/web3/apps/rust-trading-bot/trader.pid
else
    echo "⚠️  未找到Rust PID文件"
fi

# 强制清理
echo
echo "🧹 强制清理残留进程..."
pkill -f "signal_forwarder.py" && echo "   清理Python进程"
pkill -f "integrated_ai_trader" && echo "   清理Rust进程"

echo
echo "✅ 系统已完全停止"
