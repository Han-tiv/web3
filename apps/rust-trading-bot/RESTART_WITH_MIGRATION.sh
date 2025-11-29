#!/bin/bash

# ═══════════════════════════════════════════════════════════
# 架构重构 - 完整重启脚本
# 包含: 停止程序 + 数据库迁移 + 重新编译 + 启动 + 验证
# ═══════════════════════════════════════════════════════════

set -e  # 遇到错误立即退出

echo "═══════════════════════════════════════════════════════════"
echo "         🎯 架构重构 V2.0 - 自动化执行脚本"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "修改内容:"
echo "  • 删除 recommend_action 字段"
echo "  • 删除信号过滤逻辑"
echo "  • 所有信号进入AI分析"
echo ""
echo "AI分工 (保持不变):"
echo "  • Gemini → 开仓分析"
echo "  • DeepSeek → 持仓管理"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""

# ─────────────────────────────────────────────────────────────
# 步骤1: 停止旧程序
# ─────────────────────────────────────────────────────────────

echo "【步骤1/6】停止旧程序..."
OLD_PID=$(ps aux | grep integrated_ai_trader | grep -v grep | awk '{print $2}' | head -1)

if [ -n "$OLD_PID" ]; then
    echo "  找到进程 PID: $OLD_PID"
    kill $OLD_PID
    echo "  等待程序优雅退出 (5秒)..."
    sleep 5
    
    # 检查是否还在运行
    if ps -p $OLD_PID > /dev/null 2>&1; then
        echo "  ⚠️  程序未退出，强制停止..."
        kill -9 $OLD_PID
        sleep 1
    fi
    echo "  ✅ 旧程序已停止"
else
    echo "  ℹ️  未找到运行中的程序"
fi

echo ""

# ─────────────────────────────────────────────────────────────
# 步骤2: 备份数据库
# ─────────────────────────────────────────────────────────────

echo "【步骤2/6】备份数据库..."

if [ ! -f "data/trading.db" ]; then
    echo "  ❌ 错误: 数据库文件不存在: data/trading.db"
    exit 1
fi

BACKUP_NAME="data/trading.db.backup_$(date +%Y%m%d_%H%M%S)"
cp data/trading.db "$BACKUP_NAME"
echo "  ✅ 备份完成: $BACKUP_NAME"

# 显示文件大小
BACKUP_SIZE=$(du -h "$BACKUP_NAME" | cut -f1)
echo "  📊 备份大小: $BACKUP_SIZE"

echo ""

# ─────────────────────────────────────────────────────────────
# 步骤3: 执行数据库迁移
# ─────────────────────────────────────────────────────────────

echo "【步骤3/6】执行数据库迁移..."

if [ ! -f "migrations/001_simplify_telegram_signals.sql" ]; then
    echo "  ❌ 错误: 迁移脚本不存在: migrations/001_simplify_telegram_signals.sql"
    echo ""
    echo "  恢复备份:"
    echo "    cp $BACKUP_NAME data/trading.db"
    exit 1
fi

echo "  执行: sqlite3 data/trading.db < migrations/001_simplify_telegram_signals.sql"

sqlite3 data/trading.db < migrations/001_simplify_telegram_signals.sql 2>&1 | head -20

if [ $? -ne 0 ]; then
    echo "  ❌ 数据库迁移失败！"
    echo ""
    echo "  恢复备份:"
    echo "    cp $BACKUP_NAME data/trading.db"
    exit 1
fi

echo "  ✅ 数据库迁移完成"

# 验证表结构
echo ""
echo "  验证新表结构:"
sqlite3 data/trading.db ".schema telegram_signals" | head -15

echo ""

# ─────────────────────────────────────────────────────────────
# 步骤4: 重新编译
# ─────────────────────────────────────────────────────────────

echo "【步骤4/6】重新编译 (Release模式)..."
echo "  这可能需要几分钟，请耐心等待..."
echo ""

cargo build --release --bin integrated_ai_trader 2>&1 | tail -30

if [ $? -ne 0 ]; then
    echo ""
    echo "  ❌ 编译失败！"
    echo ""
    echo "  查看完整错误:"
    echo "    cargo build --release --bin integrated_ai_trader"
    echo ""
    echo "  恢复数据库:"
    echo "    cp $BACKUP_NAME data/trading.db"
    exit 1
fi

echo ""
echo "  ✅ 编译成功"
echo ""

# ─────────────────────────────────────────────────────────────
# 步骤5: 启动新程序
# ─────────────────────────────────────────────────────────────

echo "【步骤5/6】启动新程序..."

# 确保日志目录存在
mkdir -p logs

# 后台启动
nohup ./target/release/integrated_ai_trader > logs/output.log 2>&1 &
NEW_PID=$!

echo "  ✅ 程序已启动"
echo "  新进程 PID: $NEW_PID"
echo ""

# ─────────────────────────────────────────────────────────────
# 步骤6: 验证启动
# ─────────────────────────────────────────────────────────────

echo "【步骤6/6】验证启动状态..."
echo "  等待5秒后检查..."
sleep 5

if ps -p $NEW_PID > /dev/null 2>&1; then
    echo "  ✅ 程序运行中"
    echo ""
    
    # 显示最新日志
    echo "  最新日志 (startup.log):"
    echo "  ─────────────────────────────────────────────────────"
    tail -20 logs/startup.log 2>/dev/null || echo "  (尚未生成日志)"
    echo "  ─────────────────────────────────────────────────────"
    echo ""
    
    # 检查Web服务器
    echo "  检查Web服务器..."
    sleep 2
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "  ✅ Web服务器响应正常"
    else
        echo "  ⚠️  Web服务器未响应 (可能还在启动中)"
    fi
    
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "         ✅ 重构完成！系统已启动"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "关键改进:"
    echo "  ✅ 删除信号过滤 - 所有信号进入AI分析"
    echo "  ✅ 简化数据库 - 只保留必要字段"
    echo "  ✅ 保持AI分工 - Gemini开仓 + DeepSeek持仓"
    echo ""
    echo "下一步:"
    echo "  1. 观察日志:  tail -f logs/startup.log"
    echo "  2. 等待新信号到来"
    echo "  3. 验证看到: '🧠 开始AI分析' (而不是'⏭️ 跳过')"
    echo ""
    echo "监控命令:"
    echo "  • 实时日志:  tail -f logs/startup.log"
    echo "  • 进程状态:  ps aux | grep integrated_ai_trader"
    echo "  • API测试:   curl http://localhost:8080/api/status"
    echo "  • 数据库:    sqlite3 data/trading.db"
    echo ""
    echo "备份位置:"
    echo "  $BACKUP_NAME"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    
else
    echo "  ❌ 程序启动失败！"
    echo ""
    echo "查看错误日志:"
    echo "  tail -50 logs/output.log"
    echo ""
    echo "恢复数据库:"
    echo "  cp $BACKUP_NAME data/trading.db"
    echo ""
    exit 1
fi
