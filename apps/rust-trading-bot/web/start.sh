#!/bin/bash
# 交易机器人Web面板启动脚本

cd "$(dirname "$0")"

echo "🚀 启动AI交易机器人Web监控面板..."

# 检查node_modules
if [ ! -d "node_modules" ]; then
    echo "📦 首次运行,安装依赖..."
    npm install
fi

# 启动开发服务器
echo "🌐 启动Vite开发服务器..."
npm run dev
