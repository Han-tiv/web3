#!/bin/bash
# 新架构启动脚本 - Python (Telethon) + Rust (AI引擎)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 启动Valuescan V2交易系统 (新架构)"
echo "   Python (Telethon) → HTTP → Rust (AI引擎)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# 步骤1: 启动Rust交易引擎
echo "📝 步骤1: 启动Rust AI交易引擎..."
cd /home/hanins/code/web3/apps/rust-trading-bot

if [ ! -f "target/release/integrated_ai_trader" ]; then
    echo "   编译Rust程序..."
    cargo build --bin integrated_ai_trader --release 2>&1 | tail -5
fi

# 启动Rust引擎
nohup ./target/release/integrated_ai_trader > trader.log 2>&1 &
RUST_PID=$!
echo $RUST_PID > trader.pid
echo "   ✅ Rust引擎已启动 (PID: $RUST_PID)"

# 等待Rust引擎完全启动
echo "   ⏳ 等待Rust引擎启动 (10秒)..."
sleep 10

# 检查Rust引擎健康状态
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "   ✅ Rust引擎健康检查通过"
else
    echo "   ⚠️  Rust引擎可能未完全启动,但继续..."
fi

echo

# 步骤2: 启动Python Telegram监控器
echo "📝 步骤2: 启动Python Telegram监控器..."
cd /home/hanins/code/web3/apps/python-telegram-monitor

source venv/bin/activate
nohup python3 signal_forwarder.py > telegram_forwarder.log 2>&1 &
PYTHON_PID=$!
echo $PYTHON_PID > telegram.pid
echo "   ✅ Python监控器已启动 (PID: $PYTHON_PID)"

echo

# 显示状态
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 系统启动完成!"
echo
echo "📊 进程信息:"
echo "   Rust引擎:      PID $RUST_PID"
echo "   Python监控器:  PID $PYTHON_PID"
echo
echo "📋 日志文件:"
echo "   Rust:   /home/hanins/code/web3/apps/rust-trading-bot/trader.log"
echo "   Python: /home/hanins/code/web3/apps/python-telegram-monitor/telegram_forwarder.log"
echo
echo "🔧 常用命令:"
echo "   查看Rust日志:   tail -f /home/hanins/code/web3/apps/rust-trading-bot/trader.log"
echo "   查看Python日志: tail -f /home/hanins/code/web3/apps/python-telegram-monitor/telegram_forwarder.log"
echo "   停止系统:       bash /home/hanins/code/web3/apps/rust-trading-bot/stop_system.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "🎉 系统运行中,请使用上述命令查看日志!"
