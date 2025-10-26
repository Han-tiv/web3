#!/bin/bash
# ================================
# Web3项目手动部署脚本 (简化版)
# ================================
# 目标服务器: 47.79.146.30
# 用户: hanins
# 密码: hanzhikun

echo "🚀 Web3项目手动部署指南"
echo "=================================="

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}📋 部署目标:${NC}"
echo "  🖥️  服务器: 47.79.146.30"
echo "  👤 用户: hanins"
echo "  🔑 密码: hanzhikun"
echo "  📁 目标: ~/code/web3"

echo ""
echo -e "${YELLOW}🧹 创建部署包 (排除不必要文件)...${NC}"

# 创建临时部署目录
DEPLOY_DIR="./web3_deploy_$(date +%Y%m%d_%H%M%S)"
echo -e "${BLUE}📁 创建部署目录: $DEPLOY_DIR${NC}"

# 复制项目文件，排除不需要的文件
echo -e "${YELLOW}📦 复制项目文件...${NC}"

# 创建部署目录
mkdir -p "$DEPLOY_DIR"

# 使用find和tar来复制文件，排除不需要的目录
echo "正在复制文件..."

# 复制所有文件，但排除特定目录
tar --exclude='./node_modules' \
    --exclude='./apps/*/node_modules' \
    --exclude='./target' \
    --exclude='./apps/*/target' \
    --exclude='./.git' \
    --exclude='./dist' \
    --exclude='./build' \
    --exclude='./.next' \
    --exclude='./out' \
    --exclude='./.turbo' \
    --exclude='./logs' \
    --exclude='*.log' \
    --exclude='./.cache' \
    --exclude='./.parcel-cache' \
    --exclude='./coverage' \
    --exclude='./.nyc_output' \
    --exclude='./tmp' \
    --exclude='./temp' \
    --exclude='./.tmp' \
    --exclude='./sensitive_backup_*' \
    --exclude='./.DS_Store' \
    --exclude='./Thumbs.db' \
    --exclude='*.swp' \
    --exclude='*.swo' \
    --exclude='./.vscode' \
    --exclude='./.idea' \
    -cf - . | (cd "$DEPLOY_DIR" && tar -xf -)

echo -e "${GREEN}✅ 项目文件已复制到部署目录${NC}"

# 显示部署包大小
DEPLOY_SIZE=$(du -sh "$DEPLOY_DIR" | cut -f1)
echo -e "${BLUE}📦 部署包大小: $DEPLOY_SIZE${NC}"

# 创建压缩包
ARCHIVE_NAME="web3_deploy_$(date +%Y%m%d_%H%M%S).tar.gz"
echo -e "${YELLOW}🗜️  创建压缩包: $ARCHIVE_NAME${NC}"

cd "$DEPLOY_DIR" && tar -czf "../$ARCHIVE_NAME" . && cd ..

ARCHIVE_SIZE=$(du -sh "$ARCHIVE_NAME" | cut -f1)
echo -e "${GREEN}✅ 压缩包已创建: $ARCHIVE_SIZE${NC}"

echo ""
echo -e "${BLUE}📋 手动部署步骤:${NC}"
echo "=================================="

cat << EOF

方法一: 使用SCP传输压缩包
─────────────────────────────────
1️⃣ 传输压缩包到服务器:
   scp $ARCHIVE_NAME hanins@47.79.146.30:~/
   # 输入密码: hanzhikun

2️⃣ 连接到服务器:
   ssh hanins@47.79.146.30
   # 输入密码: hanzhikun

3️⃣ 在服务器上解压:
   mkdir -p ~/code
   cd ~/code
   tar -xzf ~/$ARCHIVE_NAME
   mv web3 web3_backup_$(date +%Y%m%d) 2>/dev/null || true
   mkdir web3
   cd ~
   tar -xzf $ARCHIVE_NAME -C ~/code/web3/
   rm $ARCHIVE_NAME

方法二: 使用Git克隆 (推荐)
─────────────────────────────────
1️⃣ 连接到服务器:
   ssh hanins@47.79.146.30

2️⃣ 安装Git (如果需要):
   sudo apt update && sudo apt install -y git

3️⃣ 克隆仓库:
   mkdir -p ~/code
   cd ~/code
   git clone https://github.com/Han-tiv/web3.git

4️⃣ 复制敏感配置 (从本地):
   # 在本地执行:
   scp .env hanins@47.79.146.30:~/code/web3/.env
   scp -r apps/social-monitor/services/nitter/sessions.jsonl hanins@47.79.146.30:~/code/web3/apps/social-monitor/services/nitter/
   scp apps/kronos-defi/packages/trading-engine/.env hanins@47.79.146.30:~/code/web3/apps/kronos-defi/packages/trading-engine/

EOF

echo ""
echo -e "${YELLOW}🛠️  服务器环境设置:${NC}"
echo "=================================="

cat << 'EOF'

在服务器上执行以下步骤:

1️⃣ 更新系统:
   sudo apt update && sudo apt upgrade -y

2️⃣ 安装Node.js 18+:
   curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
   sudo apt-get install -y nodejs

3️⃣ 安装Docker:
   curl -fsSL https://get.docker.com | sh
   sudo usermod -aG docker $USER
   # 注销后重新登录生效

4️⃣ 安装Rust:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

5️⃣ 进入项目目录:
   cd ~/code/web3

6️⃣ 安装依赖:
   npm install
   cd apps/rust-trading-bot && cargo build --release

7️⃣ 配置环境变量:
   cp .env.example .env
   nano .env  # 填入真实配置

8️⃣ 启动服务:
   chmod +x start.sh
   ./start.sh

EOF

echo ""
echo -e "${GREEN}🎯 当前状态:${NC}"
echo "  ✅ 部署包已准备: $ARCHIVE_NAME"
echo "  ✅ 临时目录: $DEPLOY_DIR"
echo "  📋 请选择上述部署方法之一"

echo ""
echo -e "${BLUE}💡 建议使用方法二 (Git克隆)，更简单且能保持版本同步${NC}"

echo ""
echo -e "${YELLOW}🧹 完成后清理:${NC}"
echo "rm -rf $DEPLOY_DIR $ARCHIVE_NAME"