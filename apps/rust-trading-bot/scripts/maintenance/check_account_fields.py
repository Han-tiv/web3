#!/usr/bin/env python3
"""检查 Binance Futures API 返回的账户信息字段"""
import os
import time
import hmac
import hashlib
import requests
import json
from dotenv import load_dotenv

load_dotenv('/home/hanins/code/web3/.env')

API_KEY = os.getenv('BINANCE_API_KEY')
SECRET = os.getenv('BINANCE_SECRET')

timestamp = int(time.time() * 1000)
query_string = f'timestamp={timestamp}'
signature = hmac.new(SECRET.encode(), query_string.encode(), hashlib.sha256).hexdigest()

url = f'https://fapi.binance.com/fapi/v2/account?{query_string}&signature={signature}'
headers = {'X-MBX-APIKEY': API_KEY}

response = requests.get(url, headers=headers)
data = response.json()

print("📊 Binance Futures 账户信息完整字段：\n")
print(json.dumps(data, indent=2, ensure_ascii=False))

print("\n" + "="*50)
print("关键余额字段：")
print(f"  totalWalletBalance (总钱包余额): {data.get('totalWalletBalance', 'N/A')}")
print(f"  totalMarginBalance (总保证金余额): {data.get('totalMarginBalance', 'N/A')}")
print(f"  totalCrossWalletBalance (全仓钱包余额): {data.get('totalCrossWalletBalance', 'N/A')}")
print(f"  availableBalance (可用余额): {data.get('availableBalance', 'N/A')}")
print(f"  totalUnrealizedProfit (未实现盈亏): {data.get('totalUnrealizedProfit', 'N/A')}")
