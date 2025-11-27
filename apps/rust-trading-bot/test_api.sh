#!/bin/bash
# API测试脚本

echo "===================================="
echo "AI交易机器人 Web API 测试"
echo "===================================="
echo ""

echo "1. 健康检查:"
curl -s http://localhost:8080/health
echo -e "\n"

echo "2. 账户信息:"
curl -s http://localhost:8080/api/account | python3 -m json.tool
echo ""

echo "3. 权益历史 (最近5条):"
curl -s http://localhost:8080/api/equity-history | python3 -c "import sys, json; data=json.load(sys.stdin); print(json.dumps(data[-5:], indent=2))"
echo ""

echo "4. 当前持仓:"
curl -s http://localhost:8080/api/positions | python3 -m json.tool
echo ""

echo "5. 交易历史 (最近5条):"
curl -s http://localhost:8080/api/trades?limit=5 | python3 -m json.tool
echo ""

echo "===================================="
echo "测试完成"
echo "===================================="
echo ""
echo "前端地址: http://localhost:5174"
echo "API地址: http://localhost:8080/api/"
