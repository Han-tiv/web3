#!/usr/bin/env python3

import pyotp

def test_2fa_format():
    """测试2FA格式和代码生成"""

    provided_code = "093448"

    print("=== 2FA 格式分析 ===")
    print(f"你提供的代码: {provided_code}")
    print(f"代码长度: {len(provided_code)} (标准TOTP是6位)")

    # 检查是否是有效的6位数字
    if len(provided_code) == 6 and provided_code.isdigit():
        print("✅ 符合TOTP验证码格式 (6位数字)")
    else:
        print("❌ 不符合标准TOTP格式")

    print("\n=== 需要的2FA Secret格式 ===")
    print("脚本需要的是2FA Secret (种子密钥)，不是当前验证码")
    print("通常格式如下:")
    print("- Base32字符串: JBSWY3DPEHPK3PXP")
    print("- 长度: 通常16-32个字符")
    print("- 字符集: A-Z, 2-7")

    print("\n=== 如何获取2FA Secret ===")
    print("1. 在设置Twitter 2FA时，会显示一个二维码")
    print("2. 二维码下方通常有 'Manual Entry Key' 或类似文本")
    print("3. 这个key就是我们需要的secret")
    print("4. 或者从你的2FA应用中查看account详情")

    print(f"\n=== 测试场景 ===")
    print("如果你的2FA secret是有效的，我们可以生成验证码来验证:")

    # 示例: 如果有有效的secret
    example_secret = "JBSWY3DPEHPK3PXP"  # 示例secret
    totp = pyotp.TOTP(example_secret)
    current_code = totp.now()

    print(f"示例secret '{example_secret}' 当前生成的代码: {current_code}")
    print("\n请提供你的Twitter 2FA secret (base32格式)")

if __name__ == "__main__":
    test_2fa_format()