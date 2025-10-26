#!/bin/bash
# ================================
# Web3项目敏感文件VPS备份脚本
# ================================
# 目标VPS: 158.101.132.40
# 用户: hantiv
# 路径: ~/code/web3

set -e

echo "🔐 Web3项目敏感文件备份脚本"
echo "================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# VPS配置
VPS_HOST="158.101.132.40"
VPS_USER="hantiv"
VPS_PATH="~/code/web3"

echo -e "${BLUE}📋 检查敏感文件...${NC}"

# 敏感文件列表
SENSITIVE_FILES=(
    # 环境变量文件
    "apps/kronos-defi/packages/trading-engine/.env"

    # OAuth sessions (Twitter)
    "apps/social-monitor/services/nitter/sessions.jsonl"
    "apps/social-monitor/services/nitter/data/sessions.jsonl"
    "apps/social-monitor/services/nitter/data/test_session_1.jsonl"
    "apps/social-monitor/services/nitter/data/test_session_2.jsonl"
    "apps/social-monitor/services/nitter/data/test_session_3.jsonl"

    # 配置文件 (包含敏感信息)
    "apps/social-monitor/services/nitter/config/nitter.conf"

    # 可能的其他敏感文件
    "apps/kronos-defi/packages/ai-predictor/.env.production"
    "apps/kronos-defi/packages/ai-predictor/.env.testnet"
)

echo -e "${YELLOW}🔍 发现的敏感文件:${NC}"
existing_files=()

for file in "${SENSITIVE_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
        existing_files+=("$file")

        # 显示文件包含的敏感信息类型
        case "$file" in
            *".env"*)
                echo -e "     ${BLUE}→ 包含: API密钥、数据库连接等${NC}"
                ;;
            *"sessions.jsonl"*)
                echo -e "     ${RED}→ 包含: Twitter OAuth Token${NC}"
                ;;
            *"nitter.conf"*)
                echo -e "     ${YELLOW}→ 包含: 服务配置、密钥${NC}"
                ;;
        esac
    else
        echo "  ❌ $file (文件不存在)"
    fi
done

if [ ${#existing_files[@]} -eq 0 ]; then
    echo -e "${RED}❌ 没有找到敏感文件${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}📦 准备备份 ${#existing_files[@]} 个敏感文件${NC}"

# 创建临时备份目录
BACKUP_DIR="./sensitive_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo -e "${BLUE}📁 创建备份目录: $BACKUP_DIR${NC}"

# 复制文件到备份目录，保持目录结构
for file in "${existing_files[@]}"; do
    echo "  📄 备份: $file"

    # 创建目标目录
    target_dir="$BACKUP_DIR/$(dirname "$file")"
    mkdir -p "$target_dir"

    # 复制文件
    cp "$file" "$target_dir/"
done

echo ""
echo -e "${GREEN}✅ 本地备份完成${NC}"

# 生成传输命令
echo ""
echo -e "${YELLOW}🚀 VPS传输命令:${NC}"
echo "================================"

cat << EOF

# 方法一: 使用 rsync (推荐，增量同步)
rsync -avz --progress \\
    -e "ssh -o StrictHostKeyChecking=no" \\
    "$BACKUP_DIR/" \\
    ${VPS_USER}@${VPS_HOST}:${VPS_PATH}/sensitive_backup/

# 方法二: 使用 scp (简单上传)
scp -r "$BACKUP_DIR" \\
    ${VPS_USER}@${VPS_HOST}:${VPS_PATH}/

# 方法三: 创建加密压缩包后传输
tar -czf sensitive_files.tar.gz "$BACKUP_DIR"
scp sensitive_files.tar.gz \\
    ${VPS_USER}@${VPS_HOST}:${VPS_PATH}/

EOF

echo -e "${BLUE}🔧 执行步骤:${NC}"
echo "1. 运行上述任一命令"
echo "2. 输入VPS密码: hanzhikun"
echo "3. 验证传输完成"

echo ""
echo -e "${YELLOW}📋 验证命令 (在VPS上执行):${NC}"
cat << EOF

# 连接VPS
ssh ${VPS_USER}@${VPS_HOST}

# 查看备份文件
ls -la ${VPS_PATH}/sensitive_backup/

# 验证关键文件
cat ${VPS_PATH}/sensitive_backup/apps/kronos-defi/packages/trading-engine/.env

EOF

echo ""
echo -e "${RED}🔒 安全提醒:${NC}"
echo "  - 传输完成后删除本地备份目录"
echo "  - VPS上设置适当的文件权限 (chmod 600)"
echo "  - 定期更新VPS安全补丁"
echo "  - 考虑使用SSH密钥而不是密码认证"

echo ""
echo -e "${GREEN}🎯 备份目录位置: $BACKUP_DIR${NC}"
echo "准备就绪！请运行上述传输命令。"