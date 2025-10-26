#!/bin/bash
# Web3 Monorepo 统一启动脚本
# 使用根目录 .env 环境变量配置

set -e

echo "🚀 Web3 Monorepo 启动脚本"
echo "=================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查环境变量文件
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}⚠️  未找到.env文件${NC}"
    echo -e "${BLUE}📝 正在从模板创建...${NC}"
    cp .env.example .env
    echo -e "${GREEN}✅ .env文件已创建${NC}"
    echo -e "${YELLOW}请编辑.env文件，填入你的配置:${NC}"
    echo "   nano .env"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ 环境变量文件已找到${NC}"

# 加载环境变量
export $(grep -v '^#' .env | xargs)

# 主菜单
show_menu() {
    echo ""
    echo -e "${BLUE}请选择要启动的服务:${NC}"
    echo "=================================="
    echo "1)  🤖 Rust交易机器人"
    echo "2)  📱 社交媒体监控 (Docker)"
    echo "3)  💹 Kronos DeFi交易"
    echo "4)  🧪 测试Binance API连接"
    echo "5)  🐳 启动所有Docker服务"
    echo "6)  📊 查看服务状态"
    echo "7)  🛑 停止所有服务"
    echo "8)  📝 编辑环境变量"
    echo "9)  📖 查看README"
    echo "0)  🚪 退出"
    echo "=================================="
}

# 启动Rust交易机器人
start_rust_bot() {
    echo -e "${BLUE}🤖 启动Rust交易机器人...${NC}"
    cd apps/rust-trading-bot
    ./start.sh
    cd ../..
}

# 启动社交媒体监控
start_social_monitor() {
    echo -e "${BLUE}📱 启动社交媒体监控...${NC}"
    cd apps/social-monitor
    docker-compose up -d
    echo -e "${GREEN}✅ 社交监控服务已启动${NC}"
    echo "🌐 Nitter: http://localhost:${NITTER_PORT:-3001}"
    echo "📊 监控面板: http://localhost:${SOCIAL_MONITOR_PORT:-3002}"
    cd ../..
}

# 启动Kronos DeFi
start_kronos_defi() {
    echo -e "${BLUE}💹 启动Kronos DeFi交易...${NC}"
    cd apps/kronos-defi/packages/trading-engine
    ./start.sh
    cd ../../../..
}

# 测试API连接
test_api() {
    echo -e "${BLUE}🧪 测试API连接...${NC}"
    node apps/rust-trading-bot/test-binance-api.js
}

# 启动所有Docker服务
start_all_docker() {
    echo -e "${BLUE}🐳 启动所有Docker服务...${NC}"
    cd apps/social-monitor && docker-compose up -d && cd ../..
    echo -e "${GREEN}✅ Docker服务已启动${NC}"
}

# 查看服务状态
show_status() {
    echo -e "${BLUE}📊 服务状态检查...${NC}"
    echo "=================================="

    # 检查Docker服务
    echo -e "${YELLOW}Docker服务:${NC}"
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "(social-monitor)" || echo "无运行的Docker服务"

    echo ""
    echo -e "${YELLOW}端口占用:${NC}"
    netstat -tlnp 2>/dev/null | grep -E ":300[0-9]|:456[0-9]" || echo "无相关端口占用"

    echo ""
    echo -e "${YELLOW}服务URL:${NC}"
    echo "🌐 Nitter: http://localhost:${NITTER_PORT:-3001}"
    echo "📊 社交监控: http://localhost:${SOCIAL_MONITOR_PORT:-3002}"
    echo "💹 Kronos DeFi Trading API: http://localhost:${KRONOS_TRADING_PORT:-4567}"
}

# 停止所有服务
stop_all() {
    echo -e "${YELLOW}🛑 停止所有服务...${NC}"
    cd apps/social-monitor && docker-compose down && cd ../..
    echo -e "${GREEN}✅ Docker服务已停止${NC}"
}

# 编辑环境变量
edit_env() {
    echo -e "${BLUE}📝 编辑环境变量...${NC}"
    ${EDITOR:-nano} .env
}

# 查看README
show_readme() {
    echo -e "${BLUE}📖 README.md${NC}"
    echo "=================================="
    head -50 README.md
    echo ""
    echo "查看完整README: cat README.md"
}

# 主循环
while true; do
    show_menu
    read -p "请选择 [0-9]: " choice

    case $choice in
        1) start_rust_bot ;;
        2) start_social_monitor ;;
        3) start_kronos_defi ;;
        4) test_api ;;
        5) start_all_docker ;;
        6) show_status ;;
        7) stop_all ;;
        8) edit_env ;;
        9) show_readme ;;
        0)
            echo -e "${GREEN}👋 再见！${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 无效选择，请重试${NC}"
            ;;
    esac

    echo ""
    read -p "按Enter继续..." -t 3 || true
done
