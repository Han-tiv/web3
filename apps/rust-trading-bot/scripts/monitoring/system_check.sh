#!/usr/bin/env bash

# ============================================
# rust-trading-bot 系统诊断与优化脚本
# 功能：环境检测、进程/端口检查、API 连通性、日志分析、智能建议
# ============================================

set -uo pipefail

# ---------- 彩色输出配置 ----------
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[36m"
BOLD="\033[1m"
RESET="\033[0m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${ENV_PATH:-${SCRIPT_DIR}/.env}"
LOG_SAMPLE_SIZE="${LOG_SAMPLE_SIZE:-100}"
ERROR_TAIL_COUNT="${ERROR_TAIL_COUNT:-10}"
LOCAL_API_URL="${LOCAL_API_URL:-http://localhost:8080/health}"
BINANCE_PING_URL="https://fapi.binance.com/fapi/v1/ping"
BINANCE_ACCOUNT_URL="https://fapi.binance.com/fapi/v2/account"
PORTS_TO_CHECK=("5173" "5174" "8080")
PROCESSES_TO_CHECK=("integrated_ai_trader" "vite")

declare -a SUGGESTIONS=()
declare -a ERROR_CODE_LIST=()

# ---------- 工具函数 ----------
section() {
    echo -e "\n${BOLD}${BLUE}==== $1 ====${RESET}"
}

ok() {
    echo -e "${GREEN}✔ $1${RESET}"
}

warn() {
    echo -e "${YELLOW}⚠ $1${RESET}"
}

err() {
    echo -e "${RED}✖ $1${RESET}"
}

add_suggestion() {
    local msg="$1"
    SUGGESTIONS+=("$msg")
}

mask_key() {
    local value="$1"
    local length=${#value}

    if (( length == 0 )); then
        echo "N/A"
        return
    fi

    if (( length <= 8 )); then
        local first="${value:0:1}"
        local last="${value: -1}"
        echo "${first}****${last}"
        return
    fi

    local prefix="${value:0:4}"
    local suffix="${value: -4}"
    echo "${prefix}****${suffix}"
}

load_env_file() {
    if [[ -f "$ENV_FILE" ]]; then
        # shellcheck disable=SC1090
        source "$ENV_FILE"
        ok "已加载环境文件：$ENV_FILE"
    else
        warn "未找到环境文件：$ENV_FILE"
    fi
}

check_required_var() {
    local var_name="$1"
    local value="${!var_name:-}"

    if [[ -n "$value" ]]; then
        ok "$var_name 已配置：$(mask_key "$value")"
    else
        err "$var_name 缺失，请在 .env 或环境变量中配置"
        add_suggestion "补齐 $var_name（建议在 ${ENV_FILE} 中配置，并确保仅部署环境可读取）"
    fi
}

check_env() {
    section "环境检查"

    if [[ -f "$ENV_FILE" ]]; then
        ok ".env 存在：$ENV_FILE"
    else
        warn ".env 不存在：$ENV_FILE"
    fi

    # 仅当变量未预先注入时才尝试加载 .env
    if { [[ -z "${BINANCE_API_KEY:-}" ]] || [[ -z "${BINANCE_SECRET_KEY:-}" ]]; } && [[ -f "$ENV_FILE" ]]; then
        load_env_file
    fi

    check_required_var "BINANCE_API_KEY"
    check_required_var "BINANCE_SECRET_KEY"
}

check_processes() {
    section "进程检查"

    for process in "${PROCESSES_TO_CHECK[@]}"; do
        if command -v pgrep >/dev/null 2>&1; then
            local result
            result="$(pgrep -a -f "$process" || true)"
            if [[ -n "$result" ]]; then
                ok "进程 $process 正在运行："
                echo "$result" | sed "s/^/   • /"
            else
                warn "进程 $process 未运行"
                add_suggestion "启动 $process 相关服务，确保交易前端与后端进程都活跃"
            fi
        else
            warn "pgrep 不可用，无法检测进程 $process"
        fi
    done
}

check_ports() {
    section "端口占用情况"

    for port in "${PORTS_TO_CHECK[@]}"; do
        local info=""
        if command -v lsof >/dev/null 2>&1; then
            info="$(lsof -iTCP -sTCP:LISTEN -Pn 2>/dev/null | awk -v p=":${port}" '$9 ~ p')"
        elif command -v ss >/dev/null 2>&1; then
            info="$(ss -ltnp 2>/dev/null | awk -v p=":${port}" '$4 ~ p')"
        elif command -v netstat >/dev/null 2>&1; then
            info="$(netstat -ltnp 2>/dev/null | awk -v p=":${port}" '$4 ~ p')"
        else
            warn "未找到 lsof/ss/netstat，无法检查端口 $port"
            continue
        fi

        if [[ -n "$info" ]]; then
            ok "端口 $port 正在监听："
            echo "$info" | sed "s/^/   • /"
        else
            warn "端口 $port 未被占用，可用于调试"
        fi
    done
}

http_check() {
    local label="$1"
    local url="$2"
    local timeout="${3:-8}"

    local response http_code body
    response="$(curl -sS --max-time "$timeout" -w "|||%{http_code}" "$url" 2>&1 || true)"

    if [[ "$response" == *"|||"* ]]; then
        http_code="${response##*|||}"
        body="${response%|||*}"
    else
        http_code="000"
        body="$response"
    fi

    if [[ "$http_code" == "200" ]]; then
        ok "$label (${url}) 正常，HTTP $http_code"
    elif [[ "$http_code" =~ ^[0-9]{3}$ ]]; then
        warn "$label (${url}) 返回 HTTP $http_code"
        echo "$body" | tail -n 3 | sed "s/^/   • /"
    else
        err "$label (${url}) 访问失败：$body"
    fi
}

current_millis() {
    local ts
    if ts="$(date +%s%3N 2>/dev/null)"; then
        echo "$ts"
        return
    fi

    if command -v python3 >/dev/null 2>&1; then
        python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
        return
    fi

    if command -v python >/dev/null 2>&1; then
        python - <<'PY'
import time
print(int(time.time() * 1000))
PY
        return
    fi

    local seconds
    seconds="$(date +%s)"
    echo "$((seconds * 1000))"
}

sign_query() {
    local payload="$1"
    local secret="$2"
    if ! command -v openssl >/dev/null 2>&1; then
        echo ""
        return 1
    fi
    printf "%s" "$payload" | openssl dgst -sha256 -hmac "$secret" -binary | xxd -p -c 256
}

test_binance_account() {
    local api_key="${BINANCE_API_KEY:-}"
    local secret_key="${BINANCE_SECRET_KEY:-}"

    if [[ -z "$api_key" || -z "$secret_key" ]]; then
        warn "缺少 API Key 或 Secret，跳过账户信息测试"
        return
    fi

    local timestamp query signature response http_code
    timestamp="$(current_millis)"
    query="timestamp=${timestamp}&recvWindow=5000"
    signature="$(sign_query "$query" "$secret_key" || true)"

    if [[ -z "$signature" ]]; then
        warn "无法生成签名（缺少 openssl 或 xxd），跳过账户信息测试"
        add_suggestion "安装 openssl 与 xxd 以在诊断脚本中完成账户签名验证"
        return
    fi

    response="$(curl -sS --max-time 10 -H "X-MBX-APIKEY: $api_key" \
        "${BINANCE_ACCOUNT_URL}?${query}&signature=${signature}" -w "|||%{http_code}" 2>&1 || true)"

    local body
    if [[ "$response" == *"|||"* ]]; then
        http_code="${response##*|||}"
        body="${response%|||*}"
    else
        http_code="000"
        body="$response"
    fi

    if [[ "$http_code" == "200" ]]; then
        ok "Binance 合约账户接口可用"
    else
        err "Binance 合约账户接口异常（HTTP $http_code）"
        echo "$body" | tail -n 5 | sed "s/^/   • /"

        local code_match
        code_match="$(echo "$body" | sed -nE 's/.*"code":\s*(-?[0-9]+).*/\1/p' | head -n1)"
        if [[ -n "$code_match" ]]; then
            ERROR_CODE_LIST+=("$code_match")
        fi
    fi
}

check_apis() {
    section "API 连通性测试"
    http_check "本地 Web 健康检查" "$LOCAL_API_URL" 5
    http_check "Binance FAPI Ping" "$BINANCE_PING_URL" 8
    test_binance_account
}

pick_log_file() {
    local candidates=(
        "${SCRIPT_DIR}/logs/trader.log"
        "${SCRIPT_DIR}/logs/integrated_ai_trader.log"
        "${SCRIPT_DIR}/trader.log"
        "${SCRIPT_DIR}/integrated_ai_trader.log"
    )
    for file in "${candidates[@]}"; do
        if [[ -f "$file" ]]; then
            echo "$file"
            return
        fi
    done
    echo ""
}

analyze_logs() {
    section "日志分析"
    local log_file
    log_file="$(pick_log_file)"

    if [[ -z "$log_file" ]]; then
        warn "未找到可用日志文件（尝试 logs/trader.log 等路径）"
        add_suggestion "确保 trader.log 或 integrated_ai_trader.log 可供分析，并配置日志轮转"
        return
    fi

    ok "使用日志文件：$log_file"
    local recent_errors
    recent_errors="$(tail -n "$LOG_SAMPLE_SIZE" "$log_file" 2>/dev/null | grep -i "ERROR" || true)"

    if [[ -z "$recent_errors" ]]; then
        ok "最近 ${LOG_SAMPLE_SIZE} 行未发现 ERROR 级别日志"
        return
    fi

    echo "🔢 错误类型统计（按模块）："
    echo "$recent_errors" | sed -nE 's/.* ERROR ([^]]+)].*/\1/p' | sort | uniq -c | sort -nr | sed "s/^/   • /"

    local code_stats
    code_stats="$(echo "$recent_errors" | sed -nE 's/.*"code":\s*(-?[0-9]+).*/\1/p' | sort | uniq -c | sort -nr)"

    if [[ -n "$code_stats" ]]; then
        echo "🔥 最常见错误代码："
        echo "$code_stats" | sed "s/^/   • 出现次数：/"

        mapfile -t ERROR_CODE_LIST < <(echo "$code_stats" | awk '{print $2}' | uniq)
    else
        warn "最近日志未包含标准化错误代码字段"
    fi

    echo "🧾 最近 ${ERROR_TAIL_COUNT} 条错误："
    echo "$recent_errors" | tail -n "$ERROR_TAIL_COUNT" | sed "s/^/   • /"
}

public_ip_info() {
    if ! command -v curl >/dev/null 2>&1; then
        echo "无法获取（curl 不可用）"
        return
    fi
    curl -s --max-time 4 https://api.ipify.org || echo "无法获取公网 IP"
}

suggestions() {
    section "系统建议"

    local unique_codes=()
    if [[ "${#ERROR_CODE_LIST[@]}" -gt 0 ]]; then
        # 去重保留顺序
        declare -A seen=()
        for code in "${ERROR_CODE_LIST[@]}"; do
            if [[ -z "${seen[$code]:-}" ]]; then
                unique_codes+=("$code")
                seen["$code"]=1
            fi
        done
    fi

    for code in "${unique_codes[@]}"; do
        case "$code" in
            -2015)
                add_suggestion "错误码 -2015：检查 API Key 权限（需要开通合约 & 读写权限），同时核对白名单 IP"
                ;;
            -2014)
                add_suggestion "错误码 -2014：API Key 可能损坏或被重置，请在 Binance 重新创建并更新到 .env"
                ;;
            -1021)
                add_suggestion "错误码 -1021：服务器时间偏差过大，校准系统时间或缩短 recvWindow"
                ;;
            -1003)
                add_suggestion "错误码 -1003：触发速率限制，添加退避重试或申请更高频率权限"
                ;;
            -1105)
                add_suggestion "错误码 -1105：参数不合法，确认请求体与交易对配置"
                ;;
            *)
                add_suggestion "错误码 ${code}：参考 https://binance-docs.github.io/apidocs/futures/cn/#error-codes 获取官方解释"
                ;;
        esac
    done

    local pub_ip
    pub_ip="$(public_ip_info)"
    echo "🌐 当前公网 IP：$pub_ip"
    echo "   • 将上述 IP 添加进 Binance API 白名单后再执行账户接口"

    echo "🔐 权限配置建议："
    echo "   • 勾选 Futures/合约权限，并同时勾选读取与交易权限"
    echo "   • 若使用子账户 API，请在母账户侧赋权并确认 IP 白名单同步"

    if [[ "${#SUGGESTIONS[@]}" -gt 0 ]]; then
        echo "💡 诊断建议："
        for suggestion in "${SUGGESTIONS[@]}"; do
            echo "   • $suggestion"
        done
    else
        ok "未发现需要额外处理的风险，保持现有配置即可"
    fi
}

main() {
    check_env
    check_processes
    check_ports
    check_apis
    analyze_logs
    suggestions
}

main "$@"
