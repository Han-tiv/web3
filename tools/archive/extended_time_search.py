#!/usr/bin/env python3

import base64
import urllib.parse
import pyotp
import time

def extended_time_search():
    """扩大时间窗口搜索，检查是否有时间同步问题"""

    uri = "otpauth-migration://offline?data=CjsKCjMn33IjwfdDLKQSCUAweEhhbnRpdhoHVHdpdHRlciABKAEwAkITZjczOWRlMTc1NTMxMzE0NzUxORACGAEgAA%3D%3D"
    target_code = "380427"

    # 解析并解码
    parsed = urllib.parse.urlparse(uri)
    params = urllib.parse.parse_qs(parsed.query)
    data = params['data'][0]
    decoded = base64.b64decode(data + '==')

    print(f"=== 扩大时间窗口搜索验证码 {target_code} ===")

    # 从protobuf中提取所有可能的secret
    potential_secrets = []

    # 基于已知的Google Authenticator格式查找
    for i in range(len(decoded) - 9):
        # 10字节secret
        chunk_10 = decoded[i:i+10]
        # 20字节secret
        if i + 20 <= len(decoded):
            chunk_20 = decoded[i:i+20]

            for chunk_size, chunk in [(10, chunk_10), (20, chunk_20)]:
                try:
                    # 检查是否看起来像随机bytes
                    if len(set(chunk)) > chunk_size // 3:  # 有足够的熵
                        secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
                        potential_secrets.append(secret_b32)
                except:
                    continue

    # 去重
    unique_secrets = list(set(potential_secrets))
    print(f"找到 {len(unique_secrets)} 个唯一的可能secret")

    current_time = int(time.time())
    found_matches = []

    # 扩大到前后60个时间窗口 (30分钟)
    for i, secret in enumerate(unique_secrets[:20]):  # 只检查前20个最可能的
        try:
            totp = pyotp.TOTP(secret)

            print(f"检查 {i+1}/20: {secret[:15]}...")

            # 检查前后60个时间窗口 (30分钟)
            for time_offset in range(-60, 61):
                test_time = current_time + (time_offset * 30)
                code = totp.at(test_time)

                if code == target_code:
                    time_str = time.strftime('%H:%M:%S', time.localtime(test_time))
                    minutes_offset = time_offset * 30 // 60

                    print(f"🎯 找到匹配!")
                    print(f"Secret: {secret}")
                    print(f"验证码: {code}")
                    print(f"时间: {time_str} (偏移 {minutes_offset} 分钟)")

                    found_matches.append({
                        'secret': secret,
                        'code': code,
                        'time_offset': time_offset * 30,
                        'time_str': time_str
                    })

        except Exception as e:
            continue

    if found_matches:
        print(f"\n✅ 找到 {len(found_matches)} 个匹配的secret:")
        for match in found_matches:
            print(f"Secret: {match['secret']}")
            print(f"时间偏移: {match['time_offset']}秒 ({match['time_offset']//60}分钟)")

            # 验证当前时间的验证码
            totp = pyotp.TOTP(match['secret'])
            current_code = totp.now()
            print(f"当前验证码: {current_code}")
            print()
    else:
        print(f"\n❌ 扩大时间窗口后仍未找到匹配")

        # 显示一些接近的验证码
        print(f"\n检查是否有接近的验证码:")
        target_num = int(target_code)

        for i, secret in enumerate(unique_secrets[:10]):
            try:
                totp = pyotp.TOTP(secret)
                current_code = totp.now()
                code_num = int(current_code)
                diff = abs(code_num - target_num)

                if diff < 50000:  # 显示相对接近的
                    print(f"{current_code} (差距 {diff}) - {secret[:20]}...")

            except:
                continue

    return found_matches

if __name__ == "__main__":
    results = extended_time_search()

    if results:
        print(f"\n🎉 可以使用的Twitter 2FA Secret:")
        print(f"Secret: {results[0]['secret']}")

        # 创建sessions.jsonl需要的格式
        print(f"\n可以用这个secret运行:")
        print(f"python get_session.py 0xHantiv hanzhikun176 {results[0]['secret']} ../sessions.jsonl y")
    else:
        print(f"\n需要重新检查:")
        print("1. OTP migration URI是否完整")
        print("2. 380427是否是最新的验证码")
        print("3. 或者直接从Google Authenticator重新导出")