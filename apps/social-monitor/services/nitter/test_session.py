#!/usr/bin/env python3
"""
测试Twitter session token格式
分析auth_token是否可以转换为Nitter需要的格式
"""

import json
import requests

def analyze_auth_token(auth_token):
    """分析提供的auth_token的格式和用途"""
    print(f"分析auth_token: {auth_token}")
    print(f"长度: {len(auth_token)} 字符")
    print(f"格式: {'十六进制' if all(c in '0123456789abcdefABCDEF' for c in auth_token) else '混合格式'}")

    # 检查是否是标准的Twitter auth_token格式（通常32-40位十六进制）
    if len(auth_token) >= 32 and len(auth_token) <= 40 and all(c in '0123456789abcdef' for c in auth_token):
        print("✅ 符合Twitter web session auth_token格式")
    else:
        print("❌ 不符合标准Twitter auth_token格式")

def create_nitter_session_format(auth_token):
    """尝试创建Nitter兼容的session格式"""
    # Nitter实际上可能支持多种session格式
    # 让我们创建几种可能的格式

    formats = [
        # 标准OAuth格式（Nitter官方要求）
        {
            "oauth_token": auth_token,
            "oauth_token_secret": "unknown"  # 需要配对的secret
        },

        # 可能的cookie-based格式
        {
            "auth_token": auth_token,
            "ct0": "unknown",  # CSRF token
            "session": "active"
        },

        # 简化格式
        {
            "token": auth_token,
            "type": "auth_token"
        }
    ]

    return formats

def main():
    auth_token = "0f1cad0aed50a6d8917986563695d360d5d9118"

    print("=== Twitter Session Token 分析 ===\n")

    analyze_auth_token(auth_token)

    print("\n=== 可能的Nitter Session格式 ===\n")

    formats = create_nitter_session_format(auth_token)

    for i, fmt in enumerate(formats, 1):
        print(f"格式 {i}:")
        print(json.dumps(fmt, indent=2))
        print()

        # 保存到测试文件
        filename = f"data/test_session_{i}.jsonl"
        with open(filename, 'w') as f:
            f.write(json.dumps(fmt) + '\n')
        print(f"已保存到: {filename}")
        print("-" * 40)

if __name__ == "__main__":
    main()