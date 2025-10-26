#!/bin/bash
# Web3 项目安全检查脚本
# 用途: 检查敏感文件保护状态
# 执行: ./scripts/security_check.sh

set -e

PROJECT_ROOT="/home/hanins/code/web3"
cd "$PROJECT_ROOT"

echo "🔐 Web3 项目安全检查"
echo "═══════════════════════════════════════════"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ISSUES=0

# 1. 检查 .env 文件权限
echo "1️⃣  检查 .env 文件权限..."
if [ -f ".env" ]; then
    PERM=$(stat -c %a .env 2>/dev/null || stat -f %A .env 2>/dev/null)
    if [ "$PERM" = "600" ]; then
        echo -e "   ${GREEN}✅ .env 权限安全: $PERM${NC}"
    else
        echo -e "   ${YELLOW}⚠️  .env 权限不安全: $PERM${NC}"
        echo "      建议执行: chmod 600 .env"
        ISSUES=$((ISSUES + 1))
    fi
else
    echo -e "   ${YELLOW}⚠️  .env 文件不存在${NC}"
fi
echo ""

# 2. 检查 Git 忽略状态
echo "2️⃣  检查敏感文件 Git 状态..."
SENSITIVE_FILES=(
    ".env"
    "apps/social-monitor/services/nitter/sessions.jsonl"
    "apps/social-monitor/services/nitter/data/sessions.jsonl"
)

for file in "${SENSITIVE_FILES[@]}"; do
    if [ -f "$file" ]; then
        if git check-ignore "$file" > /dev/null 2>&1; then
            echo -e "   ${GREEN}✅ $file 已被忽略${NC}"
        else
            echo -e "   ${RED}❌ $file 未被忽略!${NC}"
            ISSUES=$((ISSUES + 1))
        fi
    fi
done
echo ""

# 3. 检查是否有未提交的敏感文件
echo "3️⃣  检查 Git 暂存区..."
STAGED_SENSITIVE=$(git status --porcelain | grep -E "(\.env$|\.key|\.pem|secret|password)" | wc -l)
if [ "$STAGED_SENSITIVE" -gt 0 ]; then
    echo -e "   ${RED}❌ 发现 $STAGED_SENSITIVE 个敏感文件在暂存区!${NC}"
    git status --porcelain | grep -E "(\.env$|\.key|\.pem|secret|password)"
    ISSUES=$((ISSUES + 1))
else
    echo -e "   ${GREEN}✅ 暂存区无敏感文件${NC}"
fi
echo ""

# 4. 检查 Git 历史
echo "4️⃣  检查 .env 历史记录..."
if git log --all --full-history -- .env 2>/dev/null | grep -q "commit"; then
    echo -e "   ${RED}❌ .env 曾被提交到 Git!${NC}"
    echo "      请立即执行清理脚本移除历史记录"
    ISSUES=$((ISSUES + 1))
else
    echo -e "   ${GREEN}✅ .env 从未被提交${NC}"
fi
echo ""

# 5. 检查配置文件
echo "5️⃣  检查配置文件..."
if [ -f ".env.example" ]; then
    echo -e "   ${GREEN}✅ .env.example 存在${NC}"
    # 检查是否包含真实密钥
    if grep -qE "(sk-[a-zA-Z0-9]{32,}|[0-9a-f]{64})" .env.example 2>/dev/null; then
        echo -e "   ${RED}❌ .env.example 可能包含真实密钥!${NC}"
        ISSUES=$((ISSUES + 1))
    else
        echo -e "   ${GREEN}✅ .env.example 无真实密钥${NC}"
    fi
else
    echo -e "   ${YELLOW}⚠️  .env.example 不存在${NC}"
fi
echo ""

# 6. 检查 .gitignore 配置
echo "6️⃣  检查 .gitignore 规则..."
REQUIRED_PATTERNS=(
    ".env"
    "*.log"
    "sessions.jsonl"
    "target/"
    "node_modules/"
)

MISSING=0
for pattern in "${REQUIRED_PATTERNS[@]}"; do
    if grep -q "^${pattern}\$" .gitignore 2>/dev/null || grep -q "^${pattern}" .gitignore 2>/dev/null; then
        echo -e "   ${GREEN}✅ $pattern${NC}"
    else
        echo -e "   ${YELLOW}⚠️  缺少: $pattern${NC}"
        MISSING=$((MISSING + 1))
    fi
done

if [ $MISSING -gt 0 ]; then
    ISSUES=$((ISSUES + 1))
fi
echo ""

# 7. 查找可能的密钥泄露
echo "7️⃣  扫描可能的密钥泄露..."
LEAKED=0

# 检查代码中是否有硬编码的密钥模式
if grep -r -E "(api[_-]?key|secret|password|token)\s*=\s*['\"][^'\"]+" --include="*.rs" --include="*.js" --include="*.ts" --exclude-dir=node_modules --exclude-dir=target . 2>/dev/null | grep -v "example" | grep -v "your_" | head -1 > /dev/null; then
    echo -e "   ${RED}❌ 发现可能的硬编码密钥!${NC}"
    LEAKED=$((LEAKED + 1))
else
    echo -e "   ${GREEN}✅ 无硬编码密钥${NC}"
fi

if [ $LEAKED -gt 0 ]; then
    ISSUES=$((ISSUES + 1))
fi
echo ""

# 8. 检查敏感文件大小
echo "8️⃣  检查敏感文件大小..."
if [ -f ".env" ]; then
    SIZE=$(du -h .env | cut -f1)
    echo "   📄 .env: $SIZE"
    
    # 如果 .env 太大可能包含了不该有的内容
    SIZE_BYTES=$(stat -c%s .env 2>/dev/null || stat -f%z .env 2>/dev/null)
    if [ "$SIZE_BYTES" -gt 10240 ]; then
        echo -e "   ${YELLOW}⚠️  .env 文件较大 ($SIZE)，请检查是否包含不必要的内容${NC}"
    fi
fi
echo ""

# 总结
echo "═══════════════════════════════════════════"
if [ $ISSUES -eq 0 ]; then
    echo -e "${GREEN}✅ 安全检查通过! 无问题发现${NC}"
    echo ""
    echo "🛡️  安全等级: 优秀"
    exit 0
else
    echo -e "${RED}⚠️  发现 $ISSUES 个安全问题${NC}"
    echo ""
    echo "🛡️  安全等级: 需要改进"
    echo ""
    echo "💡 建议:"
    echo "   1. 修复上述问题"
    echo "   2. 执行: chmod 600 .env"
    echo "   3. 检查 .gitignore 配置"
    echo "   4. 验证无敏感信息在 Git 历史中"
    echo ""
    exit 1
fi
