#!/bin/bash

# ======================================
# Position Rebalance Trader 启动脚本
# ======================================
# 功能：双线程AI交易系统
# 线程1：监听 Telegram 频道 → 解析币种 → 信号入队
# 线程2：每3分钟 → AI批量分析 → 仓位调整 → 交易执行
# ======================================

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_DIR="/home/hanins/code/web3/apps/rust-trading-bot"
ENV_FILE="/home/hanins/code/web3/.env"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔄 Position Rebalance Trader 启动脚本${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# 检查目录
if [ ! -d "$PROJECT_DIR" ]; then
    echo -e "${RED}❌ 错误: 项目目录不存在: $PROJECT_DIR${NC}"
    exit 1
fi

cd "$PROJECT_DIR"

# 检查环境变量文件
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ 错误: 环境变量文件不存在: $ENV_FILE${NC}"
    exit 1
fi

# 加载环境变量
echo -e "${YELLOW}📋 加载环境变量...${NC}"
set -a
source "$ENV_FILE"
set +a

# 检查必需环境变量
check_env() {
    local var_name=$1
    local var_value=$(eval echo \$$var_name)
    
    if [ -z "$var_value" ]; then
        echo -e "${RED}❌ 错误: 缺少环境变量 $var_name${NC}"
        return 1
    else
        echo -e "${GREEN}  ✓ $var_name 已设置${NC}"
        return 0
    fi
}

echo -e "${YELLOW}🔍 检查必需环境变量...${NC}"

# 检查 Telegram 配置
check_env "TELEGRAM_API_ID" || exit 1
check_env "TELEGRAM_API_HASH" || exit 1

# 检查 DeepSeek 配置
check_env "DEEPSEEK_API_KEY" || exit 1

# 检查 Binance 配置
check_env "BINANCE_API_KEY" || exit 1
check_env "BINANCE_SECRET" || exit 1

# 检查 Telegram Session
if [ ! -f "session.session" ]; then
    echo -e "${YELLOW}⚠️  警告: Telegram session 不存在${NC}"
    echo -e "${YELLOW}   请先运行登录程序：${NC}"
    echo -e "${YELLOW}   cargo run --bin get_channels${NC}"
    read -p "是否继续？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 显示配置信息
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}⚙️  配置信息${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "${YELLOW}📡 Telegram:${NC}"
echo -e "  频道ID: ${TELEGRAM_SIGNAL_CHANNEL_ID:-2254462672}"
echo -e "  频道名: ${TELEGRAM_SIGNAL_CHANNEL_NAME:-Valuescan}"

echo -e "${YELLOW}⏱️  定时配置:${NC}"
echo -e "  评估间隔: ${REBALANCE_INTERVAL_SECS:-180} 秒"
echo -e "  信号窗口: ${SIGNAL_WINDOW_SECS:-180} 秒"

echo -e "${YELLOW}🧠 AI 配置:${NC}"
echo -e "  最大并发: ${AI_MAX_CONCURRENCY:-5}"
echo -e "  调用超时: ${AI_CALL_TIMEOUT_SECS:-10} 秒"

echo -e "${YELLOW}💰 交易配置:${NC}"
echo -e "  交易所: Binance ${BINANCE_TESTNET:+[测试网]}"
echo -e "  杠杆: ${TRADE_LEVERAGE:-5}x"
echo -e "  基础仓位: ${TRADE_BASE_POSITION_USDT:-6.0} USDT"
echo -e "  最大仓位: ${TRADE_MAX_POSITION_USDT:-100.0} USDT"
echo -e "  保证金模式: ${TRADE_MARGIN_TYPE:-cross}"

echo -e "${YELLOW}🛡️  风险控制:${NC}"
echo -e "  冷却期: ${POSITION_COOLDOWN_SECS:-300} 秒"
echo -e "  周期最大调整: ${POSITION_MAX_ADJUSTMENTS:-2} 次"

# 询问用户模式
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}选择运行模式:${NC}"
echo "  1) 开发模式 (Debug, 详细日志)"
echo "  2) 生产模式 (Release, 优化性能)"
echo "  3) 仅编译 (不运行)"
read -p "请选择 [1-3]: " -n 1 -r MODE
echo

# 设置日志级别
export RUST_LOG=${RUST_LOG:-info}
export RUST_BACKTRACE=1

case $MODE in
    1)
        echo -e "${GREEN}🚀 启动开发模式...${NC}"
        echo -e "${YELLOW}日志级别: $RUST_LOG${NC}"
        echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        cargo run --bin position_rebalance_trader
        ;;
    2)
        echo -e "${GREEN}🔨 编译 Release 版本...${NC}"
        cargo build --release --bin position_rebalance_trader
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ 编译成功${NC}"
            echo -e "${GREEN}🚀 启动生产模式...${NC}"
            echo -e "${YELLOW}日志级别: $RUST_LOG${NC}"
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            ./target/release/position_rebalance_trader
        else
            echo -e "${RED}❌ 编译失败${NC}"
            exit 1
        fi
        ;;
    3)
        echo -e "${GREEN}🔨 编译 Release 版本...${NC}"
        cargo build --release --bin position_rebalance_trader
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ 编译成功${NC}"
            echo -e "${YELLOW}可执行文件位置: ./target/release/position_rebalance_trader${NC}"
        else
            echo -e "${RED}❌ 编译失败${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}❌ 无效选择${NC}"
        exit 1
        ;;
esac

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ 程序已退出${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
