#!/bin/bash
# 智能编译脚本 - 避免与运行程序争抢资源

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
MAX_MEM_PERCENT=60  # 最大内存使用率 (%)
MAX_CPU_PERCENT=70  # 最大CPU使用率 (%)
PARALLEL_JOBS=2     # 并行编译任务数 (减少内存占用)

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🛠️  Rust Trading Bot - 智能编译脚本${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 检查是否有运行中的交易程序
check_running_traders() {
    local running_procs=$(pgrep -f "integrated_ai_trader|deepseek_trader|multi_signal_trader" || true)
    if [ -n "$running_procs" ]; then
        echo -e "${YELLOW}⚠️  检测到运行中的交易程序:${NC}"
        ps aux | grep -E "integrated_ai_trader|deepseek_trader|multi_signal_trader" | grep -v grep || true
        echo ""
        echo -e "${YELLOW}建议: 编译期间会占用大量内存和CPU,可能影响交易程序${NC}"
        echo ""
        read -p "是否继续编译? (y/N): " confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            echo -e "${RED}❌ 编译已取消${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✅ 未检测到运行中的交易程序${NC}"
    fi
}

# 检查系统资源
check_system_resources() {
    echo ""
    echo -e "${BLUE}📊 系统资源检查:${NC}"

    # 检查内存
    local mem_info=$(free | grep Mem)
    local total_mem=$(echo $mem_info | awk '{print $2}')
    local used_mem=$(echo $mem_info | awk '{print $3}')
    local mem_percent=$((used_mem * 100 / total_mem))

    echo -e "  内存使用率: ${mem_percent}% (阈值: ${MAX_MEM_PERCENT}%)"

    if [ $mem_percent -gt $MAX_MEM_PERCENT ]; then
        echo -e "${RED}⚠️  当前内存使用率过高 (${mem_percent}%),建议清理后再编译${NC}"
        echo ""
        read -p "是否继续? (y/N): " confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            echo -e "${RED}❌ 编译已取消${NC}"
            exit 1
        fi
    fi

    # 检查CPU (1分钟平均负载)
    local cpu_cores=$(nproc)
    local load_avg=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | sed 's/,//')
    local load_percent=$(echo "$load_avg * 100 / $cpu_cores" | bc)

    echo -e "  CPU负载: ${load_avg}/${cpu_cores} cores (${load_percent}%)"
    echo ""
}

# 优化编译设置
optimize_build_env() {
    echo -e "${BLUE}⚙️  优化编译环境:${NC}"

    # 限制并行编译数
    export CARGO_BUILD_JOBS=$PARALLEL_JOBS
    echo -e "  并行任务数: ${PARALLEL_JOBS}"

    # 使用增量编译
    export CARGO_INCREMENTAL=1
    echo -e "  增量编译: 启用"

    # 降低编译器优化级别 (可选,加快编译速度)
    # export CARGO_PROFILE_RELEASE_OPT_LEVEL=2

    echo ""
}

# 执行编译
run_build() {
    local target=$1
    local mode=${2:-release}

    echo -e "${BLUE}🔨 开始编译: ${target} (${mode} 模式)${NC}"
    echo ""

    if [ "$mode" == "release" ]; then
        cargo build --release --bin "$target" 2>&1 | tee /tmp/cargo_build.log
    else
        cargo build --bin "$target" 2>&1 | tee /tmp/cargo_build.log
    fi

    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✅ 编译成功: ${target}${NC}"

        # 显示二进制文件信息
        local binary_path="target/${mode}/${target}"
        if [ -f "$binary_path" ]; then
            local file_size=$(du -h "$binary_path" | cut -f1)
            echo -e "  文件大小: ${file_size}"
            echo -e "  路径: ${binary_path}"
        fi
    else
        echo ""
        echo -e "${RED}❌ 编译失败: ${target}${NC}"
        echo -e "${YELLOW}查看详细日志: /tmp/cargo_build.log${NC}"
        exit 1
    fi
}

# 编译后清理
post_build_cleanup() {
    echo ""
    echo -e "${BLUE}🧹 编译后清理:${NC}"

    # 清理增量编译缓存 (可选)
    # cargo clean -p rust-trading-bot

    # 显示target目录大小
    local target_size=$(du -sh target 2>/dev/null | cut -f1 || echo "未知")
    echo -e "  target 目录大小: ${target_size}"

    echo ""
    echo -e "${GREEN}✅ 编译流程完成${NC}"
}

# 主函数
main() {
    # 切换到项目根目录
    cd "$(dirname "$0")/.."

    check_running_traders
    check_system_resources
    optimize_build_env

    # 解析参数
    local target=${1:-integrated_ai_trader}
    local mode=${2:-release}

    run_build "$target" "$mode"
    post_build_cleanup

    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# 显示帮助信息
if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo "用法: $0 [目标程序] [模式]"
    echo ""
    echo "参数:"
    echo "  目标程序    - 要编译的二进制目标 (默认: integrated_ai_trader)"
    echo "  模式        - debug 或 release (默认: release)"
    echo ""
    echo "示例:"
    echo "  $0                              # 编译 integrated_ai_trader (release)"
    echo "  $0 deepseek_trader              # 编译 deepseek_trader (release)"
    echo "  $0 integrated_ai_trader debug   # 编译 integrated_ai_trader (debug)"
    echo ""
    exit 0
fi

main "$@"
