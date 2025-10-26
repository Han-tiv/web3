#!/bin/bash
# ================================
# 敏感文件传输到新服务器脚本
# ================================
# 目标服务器: 47.79.146.30 (hanins@)

echo "🔐 敏感文件传输脚本"
echo "================================"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

SERVER="hanins@47.79.146.30"

echo -e "${BLUE}🎯 目标服务器: $SERVER${NC}"
echo -e "${YELLOW}🔑 密码: hanzhikun${NC}"

echo ""
echo -e "${YELLOW}📋 需要传输的敏感文件:${NC}"

# 检查敏感文件是否存在
SENSITIVE_FILES=(
    "apps/kronos-defi/packages/trading-engine/.env"
    "apps/social-monitor/services/nitter/sessions.jsonl"
    "apps/social-monitor/services/nitter/data/sessions.jsonl"
    "apps/kronos-defi/packages/ai-predictor/.env.production"
    "apps/kronos-defi/packages/ai-predictor/.env.testnet"
)

existing_files=()
for file in "${SENSITIVE_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "  ✅ $file"
        existing_files+=("$file")
    else
        echo -e "  ❌ $file (不存在)"
    fi
done

if [ ${#existing_files[@]} -eq 0 ]; then
    echo -e "${RED}❌ 没有找到敏感文件${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}📤 开始传输敏感文件...${NC}"
echo "请在提示时输入密码: hanzhikun"

# 首先确保服务器上的目录结构存在
echo -e "${YELLOW}🏗️  创建服务器目录结构...${NC}"

ssh $SERVER << 'EOSSH'
echo "创建必要的目录..."
mkdir -p ~/code/web3/apps/kronos-defi/packages/trading-engine
mkdir -p ~/code/web3/apps/kronos-defi/packages/ai-predictor
mkdir -p ~/code/web3/apps/social-monitor/services/nitter/data
echo "✅ 目录结构创建完成"
EOSSH

echo ""
echo -e "${YELLOW}📤 传输敏感文件...${NC}"

# 传输每个存在的敏感文件
for file in "${existing_files[@]}"; do
    echo -e "${BLUE}📄 传输: $file${NC}"

    # 创建目标目录路径
    target_dir=$(dirname "$file")

    # 传输文件
    if scp "$file" "$SERVER:~/code/web3/$file"; then
        echo -e "${GREEN}  ✅ 成功传输: $file${NC}"
    else
        echo -e "${RED}  ❌ 传输失败: $file${NC}"
    fi
done

echo ""
echo -e "${GREEN}🎉 敏感文件传输完成！${NC}"

echo ""
echo -e "${BLUE}🔍 验证传输结果...${NC}"

# 验证传输结果
ssh $SERVER << 'EOSSH'
echo "🔍 验证敏感文件:"
cd ~/code/web3

for file in apps/kronos-defi/packages/trading-engine/.env \
            apps/social-monitor/services/nitter/sessions.jsonl \
            apps/social-monitor/services/nitter/data/sessions.jsonl \
            apps/kronos-defi/packages/ai-predictor/.env.production \
            apps/kronos-defi/packages/ai-predictor/.env.testnet; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
        echo "     大小: $(du -h "$file" | cut -f1)"
    else
        echo "  ❌ $file (缺失)"
    fi
done

echo ""
echo "🔒 设置敏感文件权限..."
find . -name ".env*" -exec chmod 600 {} \;
find . -name "sessions.jsonl" -exec chmod 600 {} \;
echo "✅ 权限设置完成"
EOSSH

echo ""
echo -e "${GREEN}✅ 所有敏感文件已安全传输到服务器！${NC}"

echo ""
echo -e "${YELLOW}🔒 安全提醒:${NC}"
echo "  - 敏感文件权限已设置为 600 (仅所有者可读写)"
echo "  - 请确保服务器安全设置正确"
echo "  - 定期更新服务器安全补丁"