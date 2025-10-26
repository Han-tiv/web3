#!/bin/bash
# Web3 项目每周清理脚本
# 用途: 清理日志、编译产物、临时文件
# 执行: ./scripts/weekly_cleanup.sh

set -e

PROJECT_ROOT="/home/hanins/code/web3"
cd "$PROJECT_ROOT"

echo "🧹 Web3 项目每周清理"
echo "═══════════════════════════════════════════"
echo ""

# 1. 清理 Rust 编译产物
echo "📦 清理 Rust 编译产物..."
if [ -d "apps/rust-trading-bot/target" ]; then
    cd apps/rust-trading-bot
    BEFORE=$(du -sh target 2>/dev/null | cut -f1)
    cargo clean 2>&1 | grep -E "Removed|removed" || echo "  已是干净状态"
    cd "$PROJECT_ROOT"
    echo "  ✅ Rust target 已清理"
else
    echo "  ✅ Rust target 目录不存在，跳过"
fi
echo ""

# 2. 清理大型日志文件 (>10MB)
echo "📄 清理大型日志文件 (>10MB)..."
LOG_COUNT=$(find . -name "*.log" -size +10M 2>/dev/null | wc -l)
if [ "$LOG_COUNT" -gt 0 ]; then
    find . -name "*.log" -size +10M -exec rm -f {} \;
    echo "  ✅ 已删除 $LOG_COUNT 个大型日志文件"
else
    echo "  ✅ 没有大型日志文件"
fi
echo ""

# 3. 清理旧日志 (7天前)
echo "📝 清理旧日志文件 (7天前)..."
OLD_LOG_COUNT=$(find ./logs -name "*.log" -mtime +7 2>/dev/null | wc -l)
if [ "$OLD_LOG_COUNT" -gt 0 ]; then
    find ./logs -name "*.log" -mtime +7 -delete 2>/dev/null
    echo "  ✅ 已删除 $OLD_LOG_COUNT 个旧日志"
else
    echo "  ✅ 没有旧日志需要清理"
fi
echo ""

# 4. 清理临时文件
echo "🗑️  清理临时文件..."
TEMP_COUNT=0

# 清理 vim 临时文件
VIM_COUNT=$(find . -name "*~" -o -name "*.swp" 2>/dev/null | wc -l)
if [ "$VIM_COUNT" -gt 0 ]; then
    find . -name "*~" -o -name "*.swp" -delete 2>/dev/null
    TEMP_COUNT=$((TEMP_COUNT + VIM_COUNT))
fi

# 清理 .tmp 文件
TMP_COUNT=$(find . -name "*.tmp" 2>/dev/null | wc -l)
if [ "$TMP_COUNT" -gt 0 ]; then
    find . -name "*.tmp" -delete 2>/dev/null
    TEMP_COUNT=$((TEMP_COUNT + TMP_COUNT))
fi

if [ "$TEMP_COUNT" -gt 0 ]; then
    echo "  ✅ 已删除 $TEMP_COUNT 个临时文件"
else
    echo "  ✅ 没有临时文件"
fi
echo ""

# 5. 显示项目大小
echo "📊 当前项目大小:"
echo ""
du -sh apps/* 2>/dev/null | awk '{printf "  %-30s %s\n", $2, $1}'
echo ""
echo "  总大小: $(du -sh . 2>/dev/null | cut -f1)"
echo ""

# 6. 完成
echo "═══════════════════════════════════════════"
echo "✅ 每周清理完成！"
echo ""
echo "💡 提示: 可以添加到 crontab 自动执行"
echo "   crontab -e"
echo "   0 2 * * 0 $PROJECT_ROOT/scripts/weekly_cleanup.sh"
echo ""
