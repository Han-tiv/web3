#!/bin/bash
# Web3项目安全检查脚本
# 用途: 部署前检查关键安全配置
# 使用: bash scripts/security_check.sh
# 版本: v1.0
# 最后更新: 2025-11-18

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="/home/hanins/code/web3"
ERRORS=0
WARNINGS=0

# 打印函数
print_header() {
    echo -e "${BLUE}========================================"
    echo -e "$1"
    echo -e "========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARNINGS++))
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    ((ERRORS++))
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# 检查文件是否存在
check_file_exists() {
    if [ -f "$1" ]; then
        return 0
    else
        return 1
    fi
}

# 检查目录是否存在
check_dir_exists() {
    if [ -d "$1" ]; then
        return 0
    else
        return 1
    fi
}

# ====== 主检查逻辑 ======

print_header "🔍 Web3项目安全检查开始..."
echo ""
echo "项目路径: $PROJECT_ROOT"
echo "检查时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# ====== 检查1: NOFX JWT密钥配置 ======
echo ""
print_header "📌 [1/5] 检查NOFX JWT密钥配置"

NOFX_CONFIG="$PROJECT_ROOT/apps/nofx/config.json"
WEAK_JWT_SECRETS=(
    "your-jwt-secret-key-change-in-production-make-it-long-and-random"
    "Qk0kAa+d0iIEzXVHXbNbm+UaN3RNabmWtH8rDWZ5OPf+4GX8pBflAHodfpbipVMyrw1fsDanHsNBjhgbDeK9Jg=="
)

if check_file_exists "$NOFX_CONFIG"; then
    print_info "检查 $NOFX_CONFIG"

    # 检查是否安装jq
    if ! command -v jq &> /dev/null; then
        print_warning "jq未安装,无法解析JSON,跳过JWT密钥检查"
        print_info "安装方法: sudo apt install jq"
    else
        JWT_IN_CONFIG=$(jq -r '.jwt_secret // empty' "$NOFX_CONFIG" 2>/dev/null || echo "")

        if [ -z "$JWT_IN_CONFIG" ] || [ "$JWT_IN_CONFIG" == "null" ]; then
            print_warning "config.json中JWT密钥为空(将从环境变量或数据库读取)"
        else
            # 检查是否为弱密钥
            FOUND_WEAK=false
            for WEAK in "${WEAK_JWT_SECRETS[@]}"; do
                if [ "$JWT_IN_CONFIG" == "$WEAK" ]; then
                    print_error "发现弱JWT密钥在 $NOFX_CONFIG"
                    print_info "  修复方法: openssl rand -base64 64"
                    FOUND_WEAK=true
                    break
                fi
            done

            if [ "$FOUND_WEAK" = false ]; then
                # 检查密钥长度
                JWT_LENGTH=${#JWT_IN_CONFIG}
                if [ $JWT_LENGTH -lt 32 ]; then
                    print_warning "JWT密钥长度不足32字符: $JWT_LENGTH"
                else
                    print_success "config.json中JWT密钥已自定义(长度: $JWT_LENGTH)"
                fi
            fi
        fi
    fi
else
    print_warning "未找到 $NOFX_CONFIG"
fi

# 检查环境变量
if [ -n "$JWT_SECRET" ]; then
    print_info "检查环境变量 JWT_SECRET"
    FOUND_WEAK_ENV=false
    for WEAK in "${WEAK_JWT_SECRETS[@]}"; do
        if [ "$JWT_SECRET" == "$WEAK" ]; then
            print_error "环境变量JWT_SECRET使用弱密钥"
            FOUND_WEAK_ENV=true
            break
        fi
    done

    if [ "$FOUND_WEAK_ENV" = false ]; then
        JWT_LENGTH=${#JWT_SECRET}
        if [ $JWT_LENGTH -lt 32 ]; then
            print_warning "环境变量JWT_SECRET长度不足32字符: $JWT_LENGTH"
        else
            print_success "环境变量JWT_SECRET已设置且安全(长度: $JWT_LENGTH)"
        fi
    fi
else
    print_warning "环境变量JWT_SECRET未设置"
fi

# ====== 检查2: crypto-trading-bot Web Token ======
echo ""
print_header "📌 [2/5] 检查crypto-trading-bot Web认证"

if [ -z "$WEB_DASHBOARD_TOKEN" ]; then
    print_warning "环境变量WEB_DASHBOARD_TOKEN未设置"
    print_info "  如果Web服务开启,建议设置Token认证"
    print_info "  生成方法: openssl rand -hex 32"
else
    TOKEN_LENGTH=${#WEB_DASHBOARD_TOKEN}
    if [ $TOKEN_LENGTH -lt 32 ]; then
        print_error "WEB_DASHBOARD_TOKEN长度不足32字符: $TOKEN_LENGTH"
    else
        print_success "WEB_DASHBOARD_TOKEN已设置(长度: $TOKEN_LENGTH)"
    fi
fi

# ====== 检查3: 敏感文件权限 ======
echo ""
print_header "📌 [3/5] 检查敏感文件权限"

ENV_FILE="$PROJECT_ROOT/.env"
if check_file_exists "$ENV_FILE"; then
    PERMS=$(stat -c "%a" "$ENV_FILE")
    if [ "$PERMS" != "600" ] && [ "$PERMS" != "400" ]; then
        print_warning ".env文件权限过宽: $PERMS (建议600或400)"
        print_info "  修复方法: chmod 600 $ENV_FILE"
    else
        print_success ".env文件权限正确: $PERMS"
    fi
else
    print_warning "未找到 $ENV_FILE"
fi

# 检查SQLite数据库文件权限
TRADING_DB="$PROJECT_ROOT/apps/rust-trading-bot/data/trading.db"
if check_file_exists "$TRADING_DB"; then
    DB_PERMS=$(stat -c "%a" "$TRADING_DB")
    if [ "$DB_PERMS" != "600" ] && [ "$DB_PERMS" != "640" ]; then
        print_warning "trading.db权限过宽: $DB_PERMS (建议600)"
        print_info "  修复方法: chmod 600 $TRADING_DB"
    else
        print_success "trading.db权限正确: $DB_PERMS"
    fi
fi

# ====== 检查4: Git敏感文件排除 ======
echo ""
print_header "📌 [4/5] 检查Git忽略配置"

GITIGNORE="$PROJECT_ROOT/.gitignore"
SENSITIVE_FILES=(".env" "*.db" "*.sqlite" "*.key" "config.json" "sessions.jsonl")

if check_file_exists "$GITIGNORE"; then
    MISSING_PATTERNS=()
    for PATTERN in "${SENSITIVE_FILES[@]}"; do
        if ! grep -q "^$PATTERN" "$GITIGNORE" && ! grep -q "/$PATTERN" "$GITIGNORE"; then
            MISSING_PATTERNS+=("$PATTERN")
        fi
    done

    if [ ${#MISSING_PATTERNS[@]} -gt 0 ]; then
        print_warning "以下模式未在.gitignore中:"
        for PATTERN in "${MISSING_PATTERNS[@]}"; do
            echo "     - $PATTERN"
        done
    else
        print_success ".gitignore配置完善"
    fi
else
    print_error "未找到.gitignore文件"
fi

# 检查Git历史是否泄露敏感信息
if check_dir_exists "$PROJECT_ROOT/.git"; then
    print_info "检查Git历史中的敏感信息..."

    # 检查.env文件
    if git -C "$PROJECT_ROOT" log --all --full-history -- .env 2>/dev/null | grep -q "commit"; then
        print_error ".env文件曾被提交到Git历史"
        print_info "  紧急修复: 参考docs/security/SECURITY_ANALYSIS.md"
    else
        print_success ".env从未提交到Git历史"
    fi

    # 检查API Key泄露
    if git -C "$PROJECT_ROOT" log --all -S "BINANCE_API_KEY=" --pretty=format:"%h %s" 2>/dev/null | grep -v ".env.example" | grep -q "BINANCE_API_KEY"; then
        print_warning "检测到可能的API Key提交记录"
    fi
fi

# ====== 检查5: 服务安全配置 ======
echo ""
print_header "📌 [5/5] 检查服务安全配置"

# 检查Web服务端口
print_info "检查活动Web服务..."
WEB_PORTS=(8080 3000 5173)
for PORT in "${WEB_PORTS[@]}"; do
    if ss -tuln 2>/dev/null | grep -q ":$PORT "; then
        LISTEN_ADDR=$(ss -tuln 2>/dev/null | grep ":$PORT " | awk '{print $5}')
        if echo "$LISTEN_ADDR" | grep -q "0.0.0.0:$PORT"; then
            print_warning "端口 $PORT 监听在所有接口(0.0.0.0),建议仅绑定127.0.0.1"
        elif echo "$LISTEN_ADDR" | grep -q "127.0.0.1:$PORT"; then
            print_success "端口 $PORT 仅监听本地(127.0.0.1)"
        fi
    fi
done

# 检查防火墙状态
if command -v ufw &> /dev/null; then
    if sudo ufw status 2>/dev/null | grep -q "Status: active"; then
        print_success "防火墙(ufw)已启用"
    else
        print_warning "防火墙(ufw)未启用"
        print_info "  启用方法: sudo ufw enable"
    fi
fi

# ====== 额外安全建议 ======
echo ""
print_header "💡 安全建议"

echo ""
echo "1. 定期轮换密钥:"
echo "   - JWT_SECRET (每季度)"
echo "   - AES_KEY (每季度)"
echo "   - 交易所API Key (每月)"
echo ""
echo "2. 启用Pre-commit Hooks:"
echo "   bash scripts/install_git_hooks.sh"
echo ""
echo "3. 集成静态分析:"
echo "   - Go项目: gosec -fmt=json -out=report.json ./..."
echo "   - Rust项目: cargo audit"
echo ""
echo "4. 配置监控告警:"
echo "   - 认证失败超过10次/分钟"
echo "   - 异常IP访问"
echo "   - 资金余额异常波动"
echo ""

# ====== 总结 ======
echo ""
print_header "📊 检查结果总结"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    print_success "安全检查通过! 发现 0 个严重问题, 0 个警告"
    echo ""
    echo "✅ 可以安全部署"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠️  安全检查完成,发现 $WARNINGS 个警告${NC}"
    echo ""
    echo "建议修复警告后再部署"
    exit 0
else
    echo -e "${RED}❌ 安全检查失败! 发现 $ERRORS 个严重问题, $WARNINGS 个警告${NC}"
    echo ""
    echo "🔴 禁止部署,请先修复严重问题!"
    echo ""
    echo "详细信息请查看:"
    echo "  - docs/security/SECURITY_ANALYSIS.md"
    echo "  - apps/nofx/security_fix_jwt_weak_key.patch"
    echo "  - apps/crypto-trading-bot/docs/SECURITY_HARDENING.md"
    exit 1
fi
