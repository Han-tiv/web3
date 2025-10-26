#!/bin/bash

# Binance账户余额查询脚本

echo "🔍 Binance账户余额查询工具"
echo ""

# 查找根目录的 .env 文件
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="$ROOT_DIR/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo "⚠️  根目录 .env 文件不存在: $ENV_FILE"
    echo ""
    echo "请在项目根目录创建 .env 文件并配置以下变量:"
    echo ""
    echo "BINANCE_API_KEY=your_api_key_here"
    echo "BINANCE_SECRET_KEY=your_secret_key_here"
    echo "BINANCE_TESTNET=true  # true=测试网, false=主网"
    exit 1
fi

# 加载根目录的环境变量（只提取键值对，过滤注释）
set -a
source <(grep -v '^#' "$ENV_FILE" | grep -E '^[A-Z_]+=')
set +a

# 检查必需的环境变量
if [ -z "$BINANCE_API_KEY" ] || [ -z "$BINANCE_SECRET_KEY" ]; then
    echo "❌ 缺少必需的环境变量"
    echo ""
    echo "请在 .env 文件中设置:"
    echo "  BINANCE_API_KEY=..."
    echo "  BINANCE_SECRET_KEY=..."
    exit 1
fi

# 显示配置信息
echo "📋 当前配置:"
echo "  网络: ${BINANCE_TESTNET:-true}"
if [ "${BINANCE_TESTNET:-true}" == "true" ]; then
    echo "  模式: 测试网 (安全)"
else
    echo "  模式: 主网 (真实资金)"
fi
echo "  API Key: ${BINANCE_API_KEY:0:8}..."
echo ""

# 编译并运行
echo "🔨 编译程序..."
cargo build --release --bin check_balance 2>&1 | grep -v "Compiling\|Finished" || true

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo "✅ 编译成功"
    echo ""
    echo "════════════════════════════════════════"
    echo ""

    # 运行程序
    cargo run --release --bin check_balance

    exit_code=$?

    echo ""

    if [ $exit_code -eq 0 ]; then
        echo "✅ 查询成功完成"
    else
        echo "❌ 查询失败 (退出码: $exit_code)"
    fi
else
    echo ""
    echo "❌ 编译失败"
    echo ""
    echo "💡 请检查:"
    echo "  1. Rust工具链是否安装 (rustc --version)"
    echo "  2. 依赖是否完整"
    echo "  3. 网络连接是否正常"
    exit 1
fi