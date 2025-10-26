#!/usr/bin/env python3

import base64
import urllib.parse
import struct
import pyotp

def parse_otpauth_migration_uri(uri):
    """解析OTP迁移URI提取2FA secrets"""

    print("=== 解析OTP迁移URI ===")
    print(f"URI: {uri}")

    # 提取data参数
    parsed = urllib.parse.urlparse(uri)
    params = urllib.parse.parse_qs(parsed.query)

    if 'data' not in params:
        print("❌ URI中没有找到data参数")
        return None

    data = params['data'][0]
    print(f"Base64 data: {data[:50]}...")

    try:
        # Base64解码
        decoded_data = base64.b64decode(data + '==')  # 添加padding
        print(f"解码后数据长度: {len(decoded_data)} bytes")

        # 这是一个protobuf格式的数据，需要手动解析
        # 基于Google Authenticator的迁移格式

        secrets = []
        i = 0

        while i < len(decoded_data):
            # 查找secret pattern
            if i + 10 < len(decoded_data):
                # 尝试提取secret (通常在特定位置)
                # Google Auth migration format analysis

                # 简化方法：查找可能的base32 secret
                for start in range(i, min(i + 50, len(decoded_data))):
                    for length in [16, 20, 32]:  # 常见secret长度
                        if start + length <= len(decoded_data):
                            try:
                                potential_secret = decoded_data[start:start + length]

                                # 检查是否可能是valid secret
                                if len(potential_secret) >= 10:
                                    # 尝试转换为base32
                                    try:
                                        secret_base32 = base64.b32encode(potential_secret).decode().rstrip('=')

                                        # 测试这个secret是否能生成valid TOTP
                                        totp = pyotp.TOTP(secret_base32)
                                        test_code = totp.now()

                                        if len(test_code) == 6 and test_code.isdigit():
                                            secrets.append({
                                                'secret_base32': secret_base32,
                                                'current_code': test_code,
                                                'raw_bytes': potential_secret.hex()
                                            })
                                            print(f"✅ 找到可能的secret: {secret_base32}")
                                            print(f"   当前生成码: {test_code}")
                                    except:
                                        continue
                            except:
                                continue
            i += 1

        return secrets

    except Exception as e:
        print(f"❌ 解析错误: {e}")
        return None

def extract_twitter_secret(uri):
    """专门提取Twitter的2FA secret"""

    # 尝试直接从URI中提取
    # otpauth-migration URIs包含encoded的OTP data

    try:
        # 解析query参数
        parsed = urllib.parse.urlparse(uri)
        params = urllib.parse.parse_qs(parsed.query)

        if 'data' in params:
            data = params['data'][0]

            # Base64解码
            decoded = base64.b64decode(data + '==')

            print("=== 原始数据分析 ===")
            print(f"解码后数据: {decoded.hex()}")
            print(f"数据长度: {len(decoded)}")

            # 查找ASCII字符串，可能包含账户信息
            ascii_parts = []
            current_str = ""

            for byte in decoded:
                if 32 <= byte <= 126:  # 可打印ASCII
                    current_str += chr(byte)
                else:
                    if len(current_str) > 2:
                        ascii_parts.append(current_str)
                    current_str = ""

            if current_str:
                ascii_parts.append(current_str)

            print("发现的ASCII字符串:")
            for part in ascii_parts:
                print(f"  - {part}")

            # 寻找可能的secret bytes
            # Twitter secrets通常是20字节 (160 bits)
            potential_secrets = []

            for i in range(len(decoded) - 19):
                chunk = decoded[i:i+20]

                # 检查是否看起来像random bytes (熵检查)
                unique_bytes = len(set(chunk))
                if unique_bytes > 10:  # 高熵，可能是secret
                    secret_b32 = base64.b32encode(chunk).decode().rstrip('=')

                    try:
                        totp = pyotp.TOTP(secret_b32)
                        code = totp.now()
                        potential_secrets.append({
                            'secret': secret_b32,
                            'code': code,
                            'position': i
                        })
                    except:
                        continue

            return potential_secrets

    except Exception as e:
        print(f"提取失败: {e}")
        return []

def main():
    uri = "otpauth-migration://offline?data=CjsKCjMn33IjwfdDLKQSCUAweEhhbnRpdhoHVHdpdHRlciABKAEwAkITZjczOWRlMTc1NTMxMzE0NzUxORACGAEgAA%3D%3D"

    print("=== Twitter 2FA Secret 提取器 ===\n")

    # 方法1: 通用OTP迁移解析
    secrets = parse_otpauth_migration_uri(uri)

    print("\n=== 专门的Twitter Secret提取 ===")
    # 方法2: 专门的Twitter提取
    twitter_secrets = extract_twitter_secret(uri)

    print(f"\n=== 结果汇总 ===")

    all_secrets = []
    if secrets:
        all_secrets.extend(secrets)

    for ts in twitter_secrets:
        all_secrets.append({
            'secret_base32': ts['secret'],
            'current_code': ts['code'],
            'method': 'twitter_specific'
        })

    if all_secrets:
        print(f"找到 {len(all_secrets)} 个可能的secret:")

        for i, secret_info in enumerate(all_secrets):
            print(f"\n候选 {i+1}:")
            print(f"  Secret: {secret_info['secret_base32']}")
            print(f"  当前代码: {secret_info['current_code']}")

            # 验证这个代码是否匹配用户之前提供的093448
            if secret_info['current_code'] == '093448':
                print(f"  🎯 这个secret生成的代码匹配你之前提供的 093448!")
                print(f"  ✅ 这很可能是正确的Twitter 2FA secret")

                return secret_info['secret_base32']

    print("\n❌ 没有找到匹配的secret")
    return None

if __name__ == "__main__":
    result = main()
    if result:
        print(f"\n🎉 最终结果: {result}")
    else:
        print("\n需要手动检查2FA应用或重新获取secret")