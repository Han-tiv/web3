#!/usr/bin/env python3
"""
临时脚本：手动同步 Binance 持仓到数据库
用于立即更新前端显示，无需等待5分钟的定时同步
"""
import os
import sys
import time
import hmac
import hashlib
import requests
import sqlite3
from datetime import datetime
from dotenv import load_dotenv

# 加载环境变量
load_dotenv('/home/hanins/code/web3/.env')

API_KEY = os.getenv('BINANCE_API_KEY')
SECRET = os.getenv('BINANCE_SECRET')
DB_PATH = '/home/hanins/code/web3/apps/rust-trading-bot/data/trading.db'

print('\n🔄 手动同步 Binance 持仓到数据库...\n')

# 1. 从 Binance 获取持仓
timestamp = int(time.time() * 1000)
query_string = f'timestamp={timestamp}'
signature = hmac.new(SECRET.encode(), query_string.encode(), hashlib.sha256).hexdigest()

url = f'https://fapi.binance.com/fapi/v2/positionRisk?{query_string}&signature={signature}'
headers = {'X-MBX-APIKEY': API_KEY}

try:
    response = requests.get(url, headers=headers, timeout=10)
    response.raise_for_status()
    data = response.json()
except Exception as e:
    print(f'❌ 获取 Binance 持仓失败: {e}')
    sys.exit(1)

# 筛选非零持仓
positions = [p for p in data if float(p.get('positionAmt', 0)) != 0]

print(f'📊 Binance 实际持仓数: {len(positions)}')

if not positions:
    print('✅ 没有持仓，无需同步\n')
    sys.exit(0)

# 2. 连接数据库
try:
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    print(f'✅ 数据库连接成功: {DB_PATH}\n')
except Exception as e:
    print(f'❌ 数据库连接失败: {e}')
    sys.exit(1)

# 3. 清空旧的持仓记录（避免重复）
try:
    cursor.execute('DELETE FROM positions')
    conn.commit()
    print('🗑️  已清空旧的持仓记录\n')
except Exception as e:
    print(f'⚠️  清空旧记录失败: {e}')

# 4. 插入新的持仓记录
for p in positions:
    amt = float(p['positionAmt'])
    side = 'LONG' if amt > 0 else 'SHORT'
    quantity = abs(amt)
    entry_price = float(p['entryPrice'])
    mark_price = float(p['markPrice'])
    unrealized_pnl = float(p['unRealizedProfit'])
    leverage = int(p['leverage'])

    # 计算盈亏百分比
    if side == 'LONG':
        pnl_pct = ((mark_price - entry_price) / entry_price) * 100.0
    else:
        pnl_pct = ((entry_price - mark_price) / entry_price) * 100.0

    entry_time = datetime.utcnow().isoformat() + 'Z'
    updated_at = datetime.utcnow().isoformat() + 'Z'

    print(f'💾 同步持仓: {p["symbol"]}')
    print(f'   方向: {side} | 数量: {quantity}')
    print(f'   入场价: ${entry_price:.4f} | 当前价: ${mark_price:.4f}')
    print(f'   盈亏: ${unrealized_pnl:.4f} ({pnl_pct:+.2f}%)')
    print(f'   杠杆: {leverage}x\n')

    try:
        cursor.execute('''
            INSERT OR REPLACE INTO positions
            (symbol, side, entry_price, current_price, quantity, pnl, pnl_pct, entry_time, leverage, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            p['symbol'],
            side,
            entry_price,
            mark_price,
            quantity,
            unrealized_pnl,
            pnl_pct,
            entry_time,
            leverage,
            updated_at
        ))
        conn.commit()
        print(f'✅ {p["symbol"]} 同步成功')
    except Exception as e:
        print(f'❌ {p["symbol"]} 同步失败: {e}')

conn.close()

print(f'\n✅ 持仓同步完成！共同步 {len(positions)} 个持仓')
print(f'🌐 刷新前端页面即可看到持仓数据\n')
