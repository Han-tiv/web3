#!/bin/bash

# Nitter API 测试脚本
echo "🔍 Nitter API 服务状态检查"
echo "=================================="

# 检查服务是否运行
if curl -s -f http://localhost:3001/health > /dev/null; then
    echo "✅ API服务运行正常"

    # 获取健康状态
    echo ""
    echo "📊 服务状态:"
    curl -s http://localhost:3001/health | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3001/health

    echo ""
    echo "📈 运行统计:"
    curl -s http://localhost:3001/stats | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3001/stats

    echo ""
    echo "⚙️ 过滤器配置:"
    curl -s http://localhost:3001/filters | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3001/filters

    echo ""
    echo "🎯 访问方式:"
    echo "- 仪表板: http://localhost:3001/dashboard"
    echo "- 健康检查: http://localhost:3001/health"
    echo "- 统计信息: http://localhost:3001/stats"
    echo "- 最新机会: http://localhost:3001/opportunities"
    echo "- 过滤配置: http://localhost:3001/filters"

else
    echo "❌ API服务未运行或无法访问"
    echo ""
    echo "请检查:"
    echo "1. 服务是否启动: npm run api"
    echo "2. 端口是否被占用: lsof -i :3001"
    echo "3. 防火墙设置是否正确"
    echo "4. Redis服务是否可用"
fi

echo ""
echo "=================================="