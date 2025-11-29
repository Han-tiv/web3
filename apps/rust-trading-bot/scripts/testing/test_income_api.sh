#!/bin/bash
# 测试Binance PAPI income API

API_KEY=$(grep BINANCE_API_KEY .env | cut -d '=' -f2)
SECRET_KEY=$(grep BINANCE_SECRET_KEY .env | cut -d '=' -f2)

# 最近12小时的时间戳
START_TIME=$(($(date +%s000) - 12*3600*1000))
END_TIME=$(date +%s000)

# 构造查询参数
QUERY="startTime=${START_TIME}&endTime=${END_TIME}&incomeType=REALIZED_PNL&timestamp=${END_TIME}"

# 生成签名
SIGNATURE=$(echo -n "${QUERY}" | openssl dgst -sha256 -hmac "${SECRET_KEY}" | awk '{print $2}')

# 调用API
curl -s -H "X-MBX-APIKEY: ${API_KEY}" \
  "https://papi.binance.com/papi/v1/um/income?${QUERY}&signature=${SIGNATURE}" | jq '.'
