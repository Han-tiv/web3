#!/usr/bin/env python3

import base64
import urllib.parse
import pyotp

def detailed_protobuf_analysis():
    """详细分析protobuf结构查找所有可能的secret"""

    uri = "otpauth-migration://offline?data=CjsKCjMn33IjwfdDLKQSCUAweEhhbnRpdhoHVHdpdHRlciABKAEwAkITZjczOWRlMTc1NTMxMzE0NzUxORACGAEgAA%3D%3D"

    parsed = urllib.parse.urlparse(uri)
    params = urllib.parse.parse_qs(parsed.query)
    data = params['data'][0]
    decoded = base64.b64decode(data + '==')

    print("=== Protobuf结构分析 ===")
    print(f"十六进制: {decoded.hex()}")
    print(f"长度: {len(decoded)} bytes")

    # 手动解析protobuf
    # Google Authenticator migration format:
    # Field 1: OtpParameters (repeated)

    hex_str = decoded.hex()

    # 查找所有可能的secret位置
    # 在protobuf中，secret通常紧跟在某些标识符后面

    print(f"\n=== 查找ASCII字符串 ===")
    ascii_parts = []
    current_str = ""

    for i, byte in enumerate(decoded):
        if 32 <= byte <= 126:  # 可打印ASCII
            current_str += chr(byte)
        else:
            if len(current_str) > 1:
                ascii_parts.append((i - len(current_str), current_str))
            current_str = ""

    if current_str:
        ascii_parts.append((len(decoded) - len(current_str), current_str))

    for pos, text in ascii_parts:
        print(f"位置 {pos}: '{text}'")

    # 基于已知的Google Authenticator格式查找
    print(f"\n=== 基于已知格式查找secret ===")

    # 查找字节序列，Google Authenticator格式中secret通常在特定位置

    # 方法1: 查找长度为10或20的可能secret
    potential_secrets = []

    # 在0x33开头的位置查找（常见的secret开始位置）
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
                        totp = pyotp.TOTP(secret_b32)
                        current_code = totp.now()

                        potential_secrets.append({
                            'secret': secret_b32,
                            'code': current_code,
                            'position': i,
                            'size': chunk_size,
                            'hex': chunk.hex()
                        })

                except:
                    continue

    # 显示唯一的secret（去重）
    seen_secrets = set()
    unique_secrets = []

    for secret_info in potential_secrets:
        if secret_info['secret'] not in seen_secrets:
            seen_secrets.add(secret_info['secret'])
            unique_secrets.append(secret_info)

    print(f"找到 {len(unique_secrets)} 个唯一的可能secret:")

    for i, secret_info in enumerate(unique_secrets):
        print(f"\n{i+1}. Secret: {secret_info['secret']}")
        print(f"   当前验证码: {secret_info['code']}")
        print(f"   位置: {secret_info['position']}")
        print(f"   长度: {secret_info['size']} bytes")
        print(f"   原始hex: {secret_info['hex']}")

    # 特别检查hex中的关键位置
    print(f"\n=== 特别检查关键位置 ===")

    # 0x0a3327df... 这个序列看起来像是secret的开始
    # 尝试不同长度的提取
    key_positions = [2, 4, 5, 6, 10, 11, 12]  # 基于hex分析的关键位置

    for pos in key_positions:
        if pos + 20 <= len(decoded):
            chunk = decoded[pos:pos+20]
            try:
                secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
                totp = pyotp.TOTP(secret_b32)
                current_code = totp.now()

                print(f"位置 {pos}: {secret_b32[:25]}... -> {current_code}")

                # 检查是否接近380427
                if abs(int(current_code) - 380427) < 10000:
                    print(f"  🔍 这个码比较接近380427!")

            except:
                continue

    # 手动检查hex中可能的secret位置
    print(f"\n=== 手动hex位置检查 ===")

    # 基于hex: 0a3b0a0a3327df7223c1f7432ca4120940307848616e7469761a0754776974746572200128013002421366373339646531373535333133313437353139100218012000

    # 0x3327df7223c1f7432ca4 看起来可能是secret
    manual_positions = [
        (4, 10),   # 3327df7223c1f7432ca4
        (4, 20),   # 从3327开始的20字节
        (2, 20),   # 从3b0a开始
        (10, 10),  # 从中间开始
    ]

    for start, length in manual_positions:
        if start + length <= len(decoded):
            chunk = decoded[start:start+length]
            try:
                secret_b32 = base64.b32encode(chunk).decode().rstrip('=')
                totp = pyotp.TOTP(secret_b32)
                current_code = totp.now()

                print(f"手动位置 {start}-{start+length}: {secret_b32} -> {current_code}")

            except:
                continue

if __name__ == "__main__":
    detailed_protobuf_analysis()