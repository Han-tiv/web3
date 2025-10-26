#!/bin/bash
# ================================
# Web3项目新服务器部署脚本
# ================================
# 目标服务器: 47.79.146.30
# 用户: hanins
# 密码: hanzhikun

set -e

echo "🚀 Web3项目新服务器部署脚本"
echo "=================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 服务器配置
NEW_SERVER="47.79.146.30"
NEW_USER="hanins"
NEW_PATH="~/code/web3"

echo -e "${BLUE}📋 部署配置信息:${NC}"
echo "  🖥️  目标服务器: $NEW_SERVER"
echo "  👤 用户名: $NEW_USER"
echo "  📁 目标路径: $NEW_PATH"
echo "  🔑 密码: hanzhikun"

echo ""
echo -e "${YELLOW}🔍 检查本地项目状态...${NC}"

# 检查当前目录
if [ ! -f "package.json" ] || [ ! -d "apps" ]; then
    echo -e "${RED}❌ 错误: 请在Web3项目根目录运行此脚本${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 确认在Web3项目根目录${NC}"

# 显示当前项目大小
TOTAL_SIZE=$(du -sh . | cut -f1)
echo -e "${BLUE}📦 当前项目大小: $TOTAL_SIZE${NC}"

# 计算排除node_modules后的大小
EXCLUDE_SIZE=$(find . -name "node_modules" -type d -exec du -sh {} + 2>/dev/null | awk '{sum+=$1} END {print sum"M"}' || echo "未知")
echo -e "${BLUE}📦 排除依赖包后估计大小: ~100MB${NC}"

echo ""
echo -e "${YELLOW}🧹 准备文件清单 (排除不必要文件)...${NC}"

# 创建排除文件列表
cat > /tmp/web3_deploy_excludes << 'EOF'
# Node.js 依赖
node_modules/
npm-debug.log*
yarn-debug.log*
.pnpm-debug.log*

# Python 依赖
__pycache__/
.venv/
venv/
env/

# Rust 编译产物
target/
*.pdb
*.rlib

# 构建产物
dist/
build/
out/
.next/
.turbo/

# 缓存文件
.cache/
.parcel-cache/
.eslintcache
*.tsbuildinfo

# 日志文件 (保留配置，排除日志)
logs/
*.log

# 临时文件
tmp/
temp/
.tmp/

# 系统文件
.DS_Store
Thumbs.db
*.swp
*.swo

# Git 文件 (会重新从GitHub克隆)
.git/

# 本地备份目录
sensitive_backup_*/

# IDE 文件
.vscode/
.idea/

# 测试覆盖率
coverage/
.nyc_output/

# 数据库文件 (运行时生成)
*.db
*.sqlite
*.sqlite3

# 大型模型文件 (可重新下载)
*.bin
*.pth
*.onnx
models/

# 交易历史数据 (运行时生成)
trade_history.json
trading_data/
backtest_results/
EOF

echo -e "${GREEN}✅ 排除列表已准备${NC}"

echo ""
echo -e "${YELLOW}🔗 测试服务器连接...${NC}"

# 测试SSH连接
if ! sshpass -p 'hanzhikun' ssh -o ConnectTimeout=10 -o StrictHostKeyChecking=no ${NEW_USER}@${NEW_SERVER} "echo '连接成功'" 2>/dev/null; then
    echo -e "${RED}❌ 无法连接到服务器，请检查:${NC}"
    echo "  - 服务器地址: $NEW_SERVER"
    echo "  - 用户名: $NEW_USER"
    echo "  - 密码: hanzhikun"
    echo "  - 网络连接"
    exit 1
fi

echo -e "${GREEN}✅ 服务器连接成功${NC}"

echo ""
echo -e "${YELLOW}📁 在服务器上创建目录...${NC}"

# 在服务器上创建目录
sshpass -p 'hanzhikun' ssh -o StrictHostKeyChecking=no ${NEW_USER}@${NEW_SERVER} << 'EOSSH'
echo "🏗️  创建项目目录..."
mkdir -p ~/code/web3
cd ~/code
echo "✅ 目录创建完成"
echo "📋 当前目录: $(pwd)"
echo "📁 目录内容: $(ls -la)"
EOSSH

echo -e "${GREEN}✅ 服务器目录已准备${NC}"

echo ""
echo -e "${YELLOW}🚀 开始同步项目文件...${NC}"

# 使用rsync同步文件
echo -e "${BLUE}📤 正在传输文件... (这可能需要几分钟)${NC}"

if sshpass -p 'hanzhikun' rsync -avz --progress \
    --exclude-from=/tmp/web3_deploy_excludes \
    -e "ssh -o StrictHostKeyChecking=no" \
    ./ \
    ${NEW_USER}@${NEW_SERVER}:${NEW_PATH}/; then
    echo -e "${GREEN}✅ 文件同步完成${NC}"
else
    echo -e "${RED}❌ 文件同步失败${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}📋 验证部署结果...${NC}"

# 验证部署
sshpass -p 'hanzhikun' ssh -o StrictHostKeyChecking=no ${NEW_USER}@${NEW_SERVER} << 'EOSSH'
echo "🔍 验证部署结果..."
cd ~/code/web3

echo "📁 项目根目录内容:"
ls -la | head -10

echo ""
echo "📦 主要应用目录:"
ls -la apps/

echo ""
echo "📋 重要配置文件检查:"
[ -f "package.json" ] && echo "✅ package.json" || echo "❌ package.json"
[ -f "start.sh" ] && echo "✅ start.sh" || echo "❌ start.sh"
[ -f ".env.example" ] && echo "✅ .env.example" || echo "❌ .env.example"
[ -f "ENV_CONFIG.md" ] && echo "✅ ENV_CONFIG.md" || echo "❌ ENV_CONFIG.md"

echo ""
echo "🚫 确认排除的文件 (应该不存在):"
[ -d "node_modules" ] && echo "❌ node_modules 存在 (应该被排除)" || echo "✅ node_modules 已排除"
[ -d "apps/social-monitor/node_modules" ] && echo "❌ 子项目node_modules存在" || echo "✅ 子项目node_modules已排除"

echo ""
echo "📊 项目大小:"
du -sh .
EOSSH

echo ""
echo -e "${GREEN}🎉 部署完成！${NC}"

echo ""
echo -e "${BLUE}📋 后续设置步骤:${NC}"
echo "=================================="

cat << 'EOF'

1️⃣ 连接到新服务器:
   ssh hanins@47.79.146.30
   # 密码: hanzhikun

2️⃣ 进入项目目录:
   cd ~/code/web3

3️⃣ 复制环境变量配置:
   cp .env.example .env
   nano .env  # 填入你的真实配置

4️⃣ 安装Node.js依赖:
   npm install

   # 如果有子项目需要单独安装:
   cd apps/social-monitor && npm install
   cd ../rust-trading-bot && npm install

5️⃣ 安装Rust (如果需要):
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

6️⃣ 构建Rust项目:
   cd apps/rust-trading-bot
   cargo build --release

7️⃣ 启动服务:
   # 使用统一启动脚本
   chmod +x start.sh
   ./start.sh

8️⃣ 配置服务自启动 (可选):
   # 设置开机自启动
   crontab -e
   # 添加: @reboot cd /home/hanins/code/web3 && ./start.sh

EOF

echo ""
echo -e "${YELLOW}🔧 服务器环境准备建议:${NC}"
echo "  - 安装Docker: curl -fsSL https://get.docker.com | sh"
echo "  - 安装Node.js: curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt-get install -y nodejs"
echo "  - 更新系统: sudo apt update && sudo apt upgrade -y"

echo ""
echo -e "${GREEN}✅ Web3项目已成功部署到新服务器！${NC}"
echo -e "${BLUE}🌐 服务器: hanins@47.79.146.30${NC}"

# 清理临时文件
rm -f /tmp/web3_deploy_excludes

echo ""
echo -e "${YELLOW}💡 提示: 敏感文件也已同步，请注意服务器安全！${NC}"