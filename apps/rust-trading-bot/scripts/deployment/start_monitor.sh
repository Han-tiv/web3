#!/bin/bash

# 启动监控脚本（后台运行）

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR_LOG="$SCRIPT_DIR/monitor.log"
PID_FILE="$SCRIPT_DIR/monitor_and_restart.pid"

cd "$SCRIPT_DIR"

# 检查监控脚本是否已在运行
if pgrep -f "monitor_and_restart.sh" > /dev/null; then
    EXISTING_PID=$(pgrep -f "monitor_and_restart.sh" | head -n 1)
    echo "⚠️  监控脚本已在运行"
    echo "   PID: $EXISTING_PID"
    echo "$EXISTING_PID" > "$PID_FILE"
    exit 0
fi

# 启动监控脚本
nohup bash monitor_and_restart.sh > "$MONITOR_LOG" 2>&1 &
MONITOR_PID=$!
echo "$MONITOR_PID" > "$PID_FILE"

echo "✅ 监控脚本已启动"
echo "📊 进程ID: $MONITOR_PID"
echo "📂 监控日志: $MONITOR_LOG"
echo ""
echo "查看监控日志: tail -f $MONITOR_LOG"
echo "停止监控: pkill -f monitor_and_restart.sh"
