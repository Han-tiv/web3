#!/bin/bash
# Rust Trading Bot 快速启动脚本
# 使用根目录统一环境变量配置

set -e

echo "🦀 Rust Trading Bot 启动脚本"
echo "================================"

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 未检测到Rust，请先安装: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust版本: $(rustc --version)"

# 检查根目录环境变量文件
ROOT_ENV="../../.env"
if [ ! -f "$ROOT_ENV" ]; then
    echo "⚠️  未找到根目录.env文件: $ROOT_ENV"
    echo "📝 请从根目录的.env.example复制并配置:"
    echo "   cd ../../ && cp .env.example .env && nano .env"
    exit 1
fi

echo "✅ 使用根目录环境变量: $ROOT_ENV"

# 加载环境变量（支持注释、空白与行内说明）
while IFS= read -r line || [ -n "$line" ]; do
    # 去掉行尾回车
    line="${line%$'\r'}"

    # 去除首尾空白
    line="${line#${line%%[!$' \t']*}}"
    line="${line%${line##*[!$' \t']}}"

    # 跳过空行与注释行
    if [[ -z "$line" || "$line" == \#* ]]; then
        continue
    fi

    # 只处理包含 '=' 的行
    if [[ "$line" != *=* ]]; then
        continue
    fi

    key="${line%%=*}"
    value="${line#*=}"

    # 去除 key/value 周围的空白
    key="${key#${key%%[!$' \t']*}}"
    key="${key%${key##*[!$' \t']}}"
    value="${value#${value%%[!$' \t']*}}"
    value="${value%${value##*[!$' \t']}}"

    # 去除行内注释（以 # 分隔，未考虑引号包裹的情况）
    if [[ "$value" == *#* ]]; then
        value="${value%%#*}"
        value="${value%${value##*[!$' \t']}}"
    fi

    # 跳过空 key
    if [[ -z "$key" ]]; then
        continue
    fi

    export "$key=$value"
done < "$ROOT_ENV"

# 验证必要的环境变量
if [ -z "$BINANCE_API_KEY" ]; then
    echo "❌ 缺少BINANCE_API_KEY，请检查根目录.env文件"
    exit 1
fi

if [ -z "$TELOXIDE_TOKEN" ]; then
    echo "❌ 缺少TELOXIDE_TOKEN，请检查根目录.env文件"
    exit 1
fi

echo "✅ 环境变量验证通过"

# 询问运行模式
SERVICE_NAME="signal_trader.service"
SYSTEMCTL_CMD="sudo systemctl"

echo ""
echo "请选择运行模式:"
echo "1) 开发模式 (快速编译，带详细日志)"
echo "2) 生产模式 (优化编译，高性能)"
echo "3) 测试模式 (仅编译检查)"
echo "4) 测试API连接"
echo "5) 启动 systemd 后台 signal_trader"
echo "6) 停止 systemd 后台 signal_trader"
echo "7) 查看 systemd 服务状态"
echo "8) 查看实时日志 (signal_trader.log)"
read -p "请选择 [1-8]: " mode

case $mode in
    1)
        echo "🚀 开发模式启动..."
        RUST_LOG=debug cargo run --bin rust-trading-bot
        ;;
    2)
        echo "🚀 生产模式编译..."
        cargo build --release
        echo "✅ 编译完成，启动程序..."
        RUST_LOG=info ./target/release/rust-trading-bot
        ;;
    3)
        echo "🔍 测试编译..."
        cargo check
        echo "✅ 编译检查通过"
        ;;
    4)
        echo "🔗 测试API连接..."
        cd ../.. && node apps/rust-trading-bot/test-binance-api.js
        ;;
    5)
        echo "🛠  启动 systemd 服务: $SERVICE_NAME"
        if ! $SYSTEMCTL_CMD daemon-reload; then
            echo "❌ systemctl daemon-reload 失败，请检查 systemd 权限"
            exit 1
        fi
        if ! $SYSTEMCTL_CMD start "$SERVICE_NAME"; then
            echo "❌ 无法启动 $SERVICE_NAME，请执行 sudo systemctl status 查看详情"
            exit 1
        fi
        $SYSTEMCTL_CMD status "$SERVICE_NAME" --no-pager
        ;;
    6)
        echo "🛑 停止 systemd 服务: $SERVICE_NAME"
        if ! $SYSTEMCTL_CMD stop "$SERVICE_NAME"; then
            echo "❌ 无法停止 $SERVICE_NAME，可能未启动"
            exit 1
        fi
        echo "✅ 已执行停止命令"
        ;;
    7)
        echo "ℹ️ 查看 systemd 服务状态: $SERVICE_NAME"
        if ! $SYSTEMCTL_CMD status "$SERVICE_NAME" --no-pager; then
            echo "❌ 无法获取状态，请确认服务是否存在"
            exit 1
        fi
        ;;
    8)
        echo "📜 实时日志（Ctrl+C 退出）"
        tail -f signal_trader.log
        ;;
    *)
        echo "❌ 无效选择"
        exit 1
        ;;
esac
