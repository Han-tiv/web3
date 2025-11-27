#!/bin/bash
# 检查当前持仓和挂单

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 查询 Binance 持仓和挂单状态"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cd /home/hanins/code/web3/apps/rust-trading-bot

# 运行 Rust 程序查询持仓
RUST_LOG=info cargo run --bin check_balance --release 2>&1 | grep -v "warning:" | tail -n 50
