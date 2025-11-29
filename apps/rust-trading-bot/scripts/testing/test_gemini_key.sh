#!/bin/bash
# 快速测试Gemini API key是否有效

API_KEY="sk-l5uDkKAsXZuJwzlG2ujHgiIzMA2E4ydpotTgdrLBB10nK37d"
BASE_URL="https://www.packyapi.com"

echo "🧪 测试 Packyapi Gemini API..."
echo "API Key: ${API_KEY:0:20}..."
echo "Endpoint: $BASE_URL/v1/chat/completions"
echo ""

curl -X POST "$BASE_URL/v1/chat/completions" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini-2.5-pro",
    "messages": [
      {
        "role": "user",
        "content": "Hello, test message"
      }
    ]
  }' | jq '.'

echo ""
echo "测试完成"
