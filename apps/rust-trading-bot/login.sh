#!/bin/bash
# Telegram 登录辅助脚本

cd /home/hanins/code/web3/apps/rust-trading-bot

echo "启动 Telegram 频道查看工具..."
echo "请准备好查看你的 Telegram 应用接收验证码"
echo ""

# 使用 expect 来自动化输入（如果用户提供验证码）
if [ -n "$1" ]; then
    echo "使用提供的验证码: $1"
    echo "$1" | ./target/release/get_channels
else
    echo "交互模式 - 请手动输入验证码"
    ./target/release/get_channels
fi
