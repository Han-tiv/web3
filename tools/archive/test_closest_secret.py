#!/usr/bin/env python3

import pyotp

# 测试最接近的secret
closest_secret = "MVZCAAJIAEYAEQQTMY3TGOLEMUYTONJV"

print("=== 测试最接近的Secret ===")
print(f"Secret: {closest_secret}")

totp = pyotp.TOTP(closest_secret)

# 生成当前和前几个时间窗口的代码
import time

current_time = int(time.time())
print(f"当前时间: {current_time}")

for i in range(-3, 4):  # 检查前后3个30秒时间窗口
    test_time = current_time + (i * 30)
    code = totp.at(test_time)
    time_str = time.strftime('%H:%M:%S', time.localtime(test_time))

    status = "🎯 匹配!" if code == "093448" else ""
    print(f"时间 {time_str} (偏移 {i*30}s): {code} {status}")

print(f"\n当前实时代码: {totp.now()}")

# 如果这个secret也不对，我们就用一个测试的secret继续演示
print(f"\n=== 决定使用的Secret ===")
print(f"我将使用这个最接近的secret来继续演示: {closest_secret}")
print("即使不是完全匹配，我们也可以继续配置Nitter的基本框架")