#!/usr/bin/env python3
"""
测试信号 API 转发
"""
import requests
import time

# 构建测试信号
test_signal = {
    'symbol': 'BTCUSDT',
    'raw_message': '📊 资金流入: BTC 💰\n价格: $98000 | 24H: +2.5%',
    'timestamp': time.time()
}

print("🧪 测试信号转发到 Rust AI 交易引擎")
print(f"   币种: {test_signal['symbol']}")
print()

try:
    response = requests.post(
        'http://localhost:8080/api/signals',
        json=test_signal,
        timeout=10
    )

    print(f"✅ HTTP状态: {response.status_code}")
    print(f"   响应: {response.json()}")

    if response.status_code == 200:
        print("\n🎉 信号传递链路测试成功！")
    else:
        print(f"\n⚠️  Rust 返回非 200 状态")

except requests.exceptions.ConnectionError:
    print("❌ 连接失败: Rust 交易引擎未运行或端口错误")
except Exception as e:
    print(f"❌ 测试失败: {e}")
