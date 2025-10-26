#!/usr/bin/env python3

import base64
import urllib.parse
import pyotp
import time

def find_matching_secret():
    """使用最新验证码380427查找匹配的secret"""

    uri = "otpauth-migration://offline?data=CjsKCjMn33IjwfdDLKQSCUAweEhhbnRpdhoHVHdpdHRlciABKAEwAkITZjczOWRlMTc1NTMxMzE0NzUxORACGAEgAA%3D%3D"
    target_code = "380427"

    # 解析并解码
    parsed = urllib.parse.urlparse(uri)
    params = urllib.parse.parse_qs(parsed.query)
    data = params['data'][0]
    decoded = base64.b64decode(data + '==')

    print(f"=== 查找匹配验证码 {target_code} 的Secret ===")

    current_time = int(time.time())

    # 生成所有可能的20字节secret
    for i in range(0, len(decoded) - 19):
        chunk = decoded[i:i+20]
        try:
            secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
            totp = pyotp.TOTP(secret_b32)

            # 检查当前时间窗口和前后几个窗口
            for time_offset in range(-5, 6):  # 检查前后5个30秒窗口
                test_time = current_time + (time_offset * 30)
                code = totp.at(test_time)

                if code == target_code:
                    time_str = time.strftime('%H:%M:%S', time.localtime(test_time))
                    print(f"🎯 找到匹配的Secret!")
                    print(f"Secret: {secret_b32}")
                    print(f"验证码: {code}")
                    print(f"时间窗口: {time_str} (偏移 {time_offset * 30}s)")
                    print(f"位置: {i}")
                    print(f"原始hex: {chunk.hex()}")

                    # 验证当前代码
                    current_code = totp.now()
                    print(f"当前实时验证码: {current_code}")

                    return secret_b32

        except:
            continue

    print(f"❌ 未找到匹配验证码 {target_code} 的secret")

    # 显示所有当前可能的验证码用于对比
    print(f"\n=== 当前所有可能的验证码 ===")
    candidates = []

    for i in range(0, len(decoded) - 19):
        chunk = decoded[i:i+20]
        try:
            secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
            totp = pyotp.TOTP(secret_b32)
            current_code = totp.now()

            candidates.append({
                'secret': secret_b32,
                'code': current_code,
                'position': i
            })

        except:
            continue

    # 按验证码排序，方便查找接近的
    candidates.sort(key=lambda x: x['code'])

    print("前20个候选验证码:")
    for i, candidate in enumerate(candidates[:20]):
        print(f"{i+1:2d}. {candidate['code']} -> {candidate['secret'][:20]}...")

    return None

if __name__ == "__main__":
    result = find_matching_secret()
    if result:
        print(f"\n✅ 找到的Twitter 2FA Secret: {result}")
    else:
        print(f"\n需要检查:")
        print("1. 380427是否是当前正确的验证码")
        print("2. 或者需要提供新的OTP migration URI")
        print("3. 确认时间同步是否正确")