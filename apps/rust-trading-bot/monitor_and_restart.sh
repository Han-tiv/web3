#!/bin/bash

# Telegram连接监控和自动重启脚本
# 用途：监控integrated_ai_trader日志，检测Telegram连续断线并自动重启

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="$SCRIPT_DIR/integrated_ai_trader.log"
PID_FILE="$SCRIPT_DIR/monitor_and_restart.pid"
RESTART_SCRIPT="$SCRIPT_DIR/stop.sh && sleep 3 && $SCRIPT_DIR/start_trader.sh"
CHECK_INTERVAL=60  # 每60秒检查一次

trap 'rm -f "$PID_FILE"' EXIT INT TERM

echo "🔍 启动Telegram连接监控..."
echo "📂 日志文件: $LOG_FILE"
echo "⏰ 检查间隔: ${CHECK_INTERVAL}秒"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

while true; do
    # 检查最近5分钟的日志中Telegram错误次数
    RECENT_ERRORS=$(tail -300 "$LOG_FILE" 2>/dev/null | grep -c "❌ Telegram连接错误")
    
    # 检查是否有"断线X分钟"的警告
    DISCONNECT_WARNING=$(tail -100 "$LOG_FILE" 2>/dev/null | grep "Telegram断线超过10分钟" | tail -1)
    
    # 检查进程是否还在运行
    PID=$(ps aux | grep integrated_ai_trader | grep -v grep | grep -v monitor | awk '{print $2}' | head -1)
    
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    
    if [ -z "$PID" ]; then
        echo "[$TIMESTAMP] ⚠️  进程未运行，尝试重启..."
        cd "$SCRIPT_DIR"
        bash stop.sh
        sleep 3
        bash start_trader.sh
        echo "[$TIMESTAMP] ✅ 已执行重启命令"
        sleep 60
        continue
    fi
    
    # 如果最近5分钟有超过50个Telegram错误（约4分钟连续断线）
    if [ "$RECENT_ERRORS" -gt 50 ]; then
        echo "[$TIMESTAMP] 🚨 检测到Telegram连续断线！"
        echo "[$TIMESTAMP]    错误次数: $RECENT_ERRORS"
        echo "[$TIMESTAMP]    执行自动重启..."
        
        cd "$SCRIPT_DIR"
        bash stop.sh
        sleep 5
        bash start_trader.sh
        
        echo "[$TIMESTAMP] ✅ 重启完成，等待5分钟观察..."
        sleep 300  # 重启后等待5分钟再继续监控
        continue
    fi
    
    # 如果有10分钟断线警告
    if [ -n "$DISCONNECT_WARNING" ]; then
        echo "[$TIMESTAMP] 🚨 检测到10分钟断线警告！"
        echo "[$TIMESTAMP]    $DISCONNECT_WARNING"
        echo "[$TIMESTAMP]    执行自动重启..."
        
        cd "$SCRIPT_DIR"
        bash stop.sh
        sleep 5
        bash start_trader.sh
        
        echo "[$TIMESTAMP] ✅ 重启完成，等待5分钟观察..."
        sleep 300
        continue
    fi
    
    # 正常情况
    if [ "$RECENT_ERRORS" -gt 0 ]; then
        echo "[$TIMESTAMP] ⚠️  检测到 $RECENT_ERRORS 个Telegram错误（未达到重启阈值）"
    else
        echo "[$TIMESTAMP] ✅ 系统运行正常 (PID: $PID)"
    fi
    
    sleep $CHECK_INTERVAL
done
