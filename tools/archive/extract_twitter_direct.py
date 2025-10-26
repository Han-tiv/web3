#!/usr/bin/env python3

import base64
import urllib.parse
import pyotp

def extract_twitter_secret_direct():
    """直接从OTP migration URI提取Twitter secret"""

    uri = "otpauth-migration://offline?data=CjsKCjMn33IjwfdDLKQSCUAweEhhbnRpdhoHVHdpdHRlciABKAEwAkITZjczOWRlMTc1NTMxMzE0NzUxORACGAEgAA%3D%3D"

    # 解析并解码
    parsed = urllib.parse.urlparse(uri)
    params = urllib.parse.parse_qs(parsed.query)
    data = params['data'][0]

    decoded = base64.b64decode(data + '==')

    print("=== 直接提取Twitter 2FA Secret ===")
    print(f"解码后的十六进制数据: {decoded.hex()}")

    # 基于Google Authenticator迁移格式的直接解析
    # 从实际数据中手动提取

    # 在decoded数据中查找特定的模式
    hex_data = decoded.hex()
    print(f"\n十六进制字符串: {hex_data}")

    # 查找可能的secret位置
    # Google Authenticator迁移通常包含secret在特定位置

    # 尝试不同的提取方法
    possible_secrets = []

    # 方法1: 查找20字节的potential secret
    for i in range(0, len(decoded) - 19):
        chunk = decoded[i:i+20]
        try:
            secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
            totp = pyotp.TOTP(secret_b32)
            code = totp.now()

            if code == '093448':  # 匹配用户提供的代码
                print(f"🎯 找到匹配的secret!")
                print(f"Secret: {secret_b32}")
                print(f"生成的代码: {code}")
                return secret_b32

            # 记录所有可能的secret
            possible_secrets.append({
                'secret': secret_b32,
                'code': code,
                'position': i,
                'raw_hex': chunk.hex()
            })

        except:
            continue

    # 如果没找到完全匹配的，显示最有可能的候选
    print(f"\n没有找到完全匹配093448的secret")
    print(f"找到 {len(possible_secrets)} 个可能的secret:")

    # 显示前几个候选
    for i, secret_info in enumerate(possible_secrets[:10]):
        print(f"\n候选 {i+1}:")
        print(f"  Position: {secret_info['position']}")
        print(f"  Secret: {secret_info['secret']}")
        print(f"  当前代码: {secret_info['code']}")
        print(f"  原始hex: {secret_info['raw_hex']}")

    # 手动分析数据结构
    print(f"\n=== 手动分析数据结构 ===")

    # 在protobuf数据中查找关键字段
    # 0x0a = 字段1 (LENGTH_DELIMITED)
    # 查找Twitter相关信息

    twitter_positions = []
    twitter_bytes = b'Twitter'

    pos = decoded.find(twitter_bytes)
    if pos != -1:
        print(f"找到'Twitter'字符串在位置: {pos}")

        # 在Twitter字符串前后查找可能的secret
        start_search = max(0, pos - 30)
        end_search = min(len(decoded), pos + 30)

        print(f"在Twitter附近查找secret (位置 {start_search} - {end_search}):")

        for i in range(start_search, end_search - 19):
            if i >= 0:
                chunk = decoded[i:i+20]
                try:
                    secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
                    if len(secret_b32) >= 16:  # 合理的secret长度
                        totp = pyotp.TOTP(secret_b32)
                        code = totp.now()
                        print(f"  位置 {i}: {secret_b32} -> {code}")

                        if code == '093448':
                            print(f"  🎯 匹配的secret!")
                            return secret_b32
                except:
                    continue

    return None

if __name__ == "__main__":
    result = extract_twitter_secret_direct()
    if result:
        print(f"\n✅ 最终Twitter 2FA Secret: {result}")
    else:
        print(f"\n❌ 未找到匹配的secret。可能需要:")
        print("1. 检查093448是否是当前正确的代码")
        print("2. 或者该迁移URI不包含Twitter的secret")
        print("3. 尝试从2FA应用中重新获取secret")