#!/usr/bin/env python3
import os
import time
import hmac
import hashlib
import requests
from dotenv import load_dotenv

# 加载环境变量
load_dotenv('/home/hanins/code/web3/.env')

API_KEY = os.getenv('BINANCE_API_KEY')
SECRET = os.getenv('BINANCE_SECRET')

timestamp = int(time.time() * 1000)
query_string = f'timestamp={timestamp}'
signature = hmac.new(SECRET.encode(), query_string.encode(), hashlib.sha256).hexdigest()

url = f'https://fapi.binance.com/fapi/v2/positionRisk?{query_string}&signature={signature}'
headers = {'X-MBX-APIKEY': API_KEY}

response = requests.get(url, headers=headers)
data = response.json()

# 筛选非零持仓
positions = [p for p in data if float(p.get('positionAmt', 0)) != 0]

print(f'\n📊 总持仓数: {len(positions)}\n')
if positions:
    for p in positions:
        amt = float(p['positionAmt'])
        side = 'LONG' if amt > 0 else 'SHORT'
        print(f"  {p['symbol']}: {abs(amt)} {side}")
        print(f"    入场价: ${p['entryPrice']}")
        print(f"    标记价: ${p['markPrice']}")
        print(f"    未实现盈亏: ${p['unRealizedProfit']}")
        print(f"    杠杆: {p['leverage']}x")
        print()
else:
    print('  ❌ 没有持仓\n')
