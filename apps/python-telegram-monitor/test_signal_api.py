#!/usr/bin/env python3
"""
测试信号 API 转发
"""
import requests
import time

# 构建测试信号
test_signal = {
    'symbol': 'BTCUSDT',
    'side': 'LONG',
    'entry_price': 98000.0,
    'stop_loss': 95000.0,
    'take_profit': 102000.0,
    'confidence': 'HIGH',
    'leverage': 10,
    'source': 'telegram_python_test',
    'timestamp': time.time(),
    'raw_message': '📊 资金流入: BTC 💰\n价格: $98000 | 24H: +2.5% | 类型: 强烈看多',
    'signal_type': '强烈看多',
    'score': 8,
    'risk_level': 'NORMAL'
}

print("🧪 测试信号转发到 Rust AI 交易引擎")
print(f"   币种: {test_signal['symbol']}")
print(f"   方向: {test_signal['side']}")
print(f"   价格: ${test_signal['entry_price']:.2f}")
print(f"   信心: {test_signal['confidence']}")
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
