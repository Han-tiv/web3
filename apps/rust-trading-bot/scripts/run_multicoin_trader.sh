#!/bin/bash
# DeepSeek AI Trading Bot - 多币种版本
# 支持 BTC, ETH, SOL, BNB, DOGE

set -e

PROJECT_ROOT="/home/hanins/code/web3/apps/rust-trading-bot"
cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════"
echo "🤖 DeepSeek AI Trading Bot - Multi-Coin"
echo "═══════════════════════════════════════════"
echo ""

# 检查环境变量文件
if [ ! -f "../../.env" ]; then
    echo "❌ 错误: 找不到 .env 文件"
    echo "   请在 /home/hanins/code/web3/.env 中配置环境变量"
    exit 1
fi

echo "✅ 环境变量文件: ../../.env"
source ../../.env

# 检查必需的环境变量
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "❌ 错误: DEEPSEEK_API_KEY 未设置"
    exit 1
fi

if [ -z "$GATE_API_KEY" ] || [ -z "$GATE_SECRET" ]; then
    echo "❌ 错误: Gate.io API 密钥未设置"
    exit 1
fi

echo "✅ 环境变量检查通过"
echo ""

# 显示支持的币种
echo "💎 支持的交易币种:"
echo "   1. BTC  - Bitcoin      (BTC/USDT)  - 推荐新手"
echo "   2. ETH  - Ethereum     (ETH/USDT)  - 平衡型"
echo "   3. XRP  - Ripple       (XRP/USDT)  - 稳定型"
echo "   4. SOL  - Solana       (SOL/USDT)  - 高波动"
echo "   5. BNB  - Binance Coin (BNB/USDT)  - 稳定型"
echo "   6. DOGE - Dogecoin     (DOGE/USDT) - 投机型"
echo ""

# 检查是否设置了 TRADING_SYMBOL
if [ -z "$TRADING_SYMBOL" ]; then
    echo "❓ 请选择交易币种 (默认: BTC):"
    echo ""
    read -p "输入币种代码 (BTC/ETH/XRP/SOL/BNB/DOGE) [BTC]: " user_symbol
    
    if [ -z "$user_symbol" ]; then
        export TRADING_SYMBOL="BTC"
    else
        export TRADING_SYMBOL="${user_symbol^^}"  # 转大写
    fi
fi

echo "✅ 选择的币种: $TRADING_SYMBOL"
echo ""

# 验证币种
case "$TRADING_SYMBOL" in
    BTC|ETH|XRP|SOL|BNB|DOGE)
        echo "✅ 币种有效"
        ;;
    *)
        echo "❌ 错误: 不支持的币种 '$TRADING_SYMBOL'"
        echo "   支持的币种: BTC, ETH, XRP, SOL, BNB, DOGE"
        exit 1
        ;;
esac

echo ""

# 编译检查
echo "🔨 检查编译状态..."
if ! cargo check --bin deepseek_trader --quiet 2>/dev/null; then
    echo "⚠️  需要重新编译..."
    cargo build --release --bin deepseek_trader
else
    echo "✅ 编译检查通过"
fi
echo ""

# 检查二进制文件
if [ ! -f "target/release/deepseek_trader" ]; then
    echo "🔨 编译 release 版本..."
    cargo build --release --bin deepseek_trader
    echo "✅ 编译完成"
fi
echo ""

# 显示币种信息
case "$TRADING_SYMBOL" in
    BTC)
        echo "📊 Bitcoin (BTC) 配置:"
        echo "   交易对: BTC/USDT"
        echo "   最小交易量: 0.0001 BTC"
        echo "   参考价格: ~$67,000"
        echo "   风险级别: 🟢 低"
        ;;
    ETH)
        echo "📊 Ethereum (ETH) 配置:"
        echo "   交易对: ETH/USDT"
        echo "   最小交易量: 0.001 ETH"
        echo "   参考价格: ~$4,000"
        echo "   风险级别: 🟡 中"
        ;;
    XRP)
        echo "📊 Ripple (XRP) 配置:"
        echo "   交易对: XRP/USDT"
        echo "   最小交易量: 1.0 XRP"
        echo "   参考价格: ~$2.50"
        echo "   风险级别: 🟡 中"
        ;;
    SOL)
        echo "📊 Solana (SOL) 配置:"
        echo "   交易对: SOL/USDT"
        echo "   最小交易量: 0.01 SOL"
        echo "   参考价格: ~$200"
        echo "   风险级别: 🟠 高"
        ;;
    BNB)
        echo "📊 Binance Coin (BNB) 配置:"
        echo "   交易对: BNB/USDT"
        echo "   最小交易量: 0.01 BNB"
        echo "   参考价格: ~$1,100"
        echo "   风险级别: 🟡 中"
        ;;
    DOGE)
        echo "📊 Dogecoin (DOGE) 配置:"
        echo "   交易对: DOGE/USDT"
        echo "   最小交易量: 1.0 DOGE"
        echo "   参考价格: ~$0.20"
        echo "   风险级别: 🔴 极高"
        ;;
esac

echo ""
echo "⚙️  交易配置:"
echo "   杠杆: 5x"
echo "   周期: 15分钟整点执行"
echo "   基础投入: 100 USDT"
echo ""

# 警告信息
echo "⚠️  重要提示:"
echo "   - 这是实盘交易，会使用真实资金"
echo "   - 加密货币波动大，可能造成损失"
echo "   - 建议先用小额测试"
echo "   - 随时可以按 Ctrl+C 停止"
echo "   - 风险自负，谨慎交易"
echo ""

# 根据币种给出特殊提示
case "$TRADING_SYMBOL" in
    DOGE)
        echo "🚨 DOGE 特别警告:"
        echo "   - DOGE 是高度投机的币种"
        echo "   - 受社交媒体影响极大"
        echo "   - AI 预测准确性较低"
        echo "   - 仅建议小额资金测试"
        echo ""
        ;;
    SOL)
        echo "⚠️  SOL 特别提示:"
        echo "   - SOL 波动较大"
        echo "   - 可能出现剧烈行情"
        echo "   - 注意设置止损"
        echo ""
        ;;
esac

read -p "确认启动 $TRADING_SYMBOL AI 交易? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ 取消启动"
    exit 0
fi

echo ""
echo "🚀 启动 DeepSeek AI Trading Bot ($TRADING_SYMBOL)..."
echo "═══════════════════════════════════════════"
echo ""

# 设置日志级别
export RUST_LOG="${RUST_LOG:-info}"

# 运行
exec ./target/release/deepseek_trader
