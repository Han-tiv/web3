#!/usr/bin/env python3

import pyotp
import time

def check_all_candidates_for_380427():
    """检查所有候选secret是否在任意时间窗口能生成380427"""

    # 从上面分析得到的所有unique secret候选
    candidates = [
        "BI5QUCRTE7PXEI6B65BSZJASBFADA6CI",
        "HMFAUMZH35ZCHQPXIMWKIEQJIAYHQSDB",
        "BIFDGJ67OIR4D52DFSSBECKAGB4EQYLO",
        "BIZSPX3SEPA7OQZMUQJASQBQPBEGC3TU",
        "GMT564RDYH3UGLFECIEUAMDYJBQW45DJ",
        "E7PXEI6B65BSZJASBFADA6CIMFXHI2LW",
        "35ZCHQPXIMWKIEQJIAYHQSDBNZ2GS5Q2",
        "OIR4D52DFSSBECKAGB4EQYLOORUXMGQH",
        "EPA7OQZMUQJASQBQPBEGC3TUNF3BUB2U",
        "YH3UGLFECIEUAMDYJBQW45DJOYNAOVDX",
        "65BSZJASBFADA6CIMFXHI2LWDIDVI53J",
        "IMWKIEQJIAYHQSDBNZ2GS5Q2A5KHO2LU",
        "FSSBECKAGB4EQYLOORUXMGQHKR3WS5DU",
        "UQJASQBQPBEGC3TUNF3BUB2UO5UXI5DF",
        "CIEUAMDYJBQW45DJOYNAOVDXNF2HIZLS",
        "BFADA6CIMFXHI2LWDIDVI53JOR2GK4RA",
        "IAYHQSDBNZ2GS5Q2A5KHO2LUORSXEIAB",
        "GB4EQYLOORUXMGQHKR3WS5DUMVZCAAJI",
        "PBEGC3TUNF3BUB2UO5UXI5DFOIQACKAB",
        "JBQW45DJOYNAOVDXNF2HIZLSEAASQAJQ",
        "MFXHI2LWDIDVI53JOR2GK4RAAEUACMAC",
        "NZ2GS5Q2A5KHO2LUORSXEIABFAATAASC",
        "ORUXMGQHKR3WS5DUMVZCAAJIAEYAEQQT",
        "NF3BUB2UO5UXI5DFOIQACKABGABEEE3G",
        "OYNAOVDXNF2HIZLSEAASQAJQAJBBGZRX",
        "DIDVI53JOR2GK4RAAEUACMACIIJWMNZT",
        "A5KHO2LUORSXEIABFAATAASCCNTDOMZZ",
        "KR3WS5DUMVZCAAJIAEYAEQQTMY3TGOLE",
        "O5UXI5DFOIQACKABGABEEE3GG4ZTSZDF",
        "NF2HIZLSEAASQAJQAJBBGZRXGM4WIZJR"
    ]

    target_code = "380427"
    current_time = int(time.time())

    print(f"=== 检查 {len(candidates)} 个候选secret是否能生成 {target_code} ===")

    found_matches = []

    for i, secret in enumerate(candidates):
        try:
            totp = pyotp.TOTP(secret)

            # 检查前后20个时间窗口 (10分钟)
            for time_offset in range(-20, 21):
                test_time = current_time + (time_offset * 30)
                code = totp.at(test_time)

                if code == target_code:
                    time_str = time.strftime('%H:%M:%S', time.localtime(test_time))
                    print(f"🎯 找到匹配!")
                    print(f"Secret: {secret}")
                    print(f"验证码: {code}")
                    print(f"时间: {time_str} (偏移 {time_offset * 30}秒)")

                    found_matches.append({
                        'secret': secret,
                        'code': code,
                        'time_offset': time_offset * 30,
                        'time_str': time_str
                    })

        except Exception as e:
            print(f"Secret {i+1} 处理错误: {e}")
            continue

    if found_matches:
        print(f"\n✅ 总共找到 {len(found_matches)} 个匹配的secret:")
        for match in found_matches:
            print(f"Secret: {match['secret']}")
            print(f"时间偏移: {match['time_offset']}秒")
    else:
        print(f"\n❌ 在所有候选中都没找到能生成 {target_code} 的secret")
        print("\n可能原因:")
        print("1. 提供的verification code 380427 不是来自这个OTP migration URI")
        print("2. 时间同步问题")
        print("3. OTP migration URI可能不完整或损坏")
        print("4. 需要重新从2FA应用导出")

        # 显示当前所有候选的验证码
        print(f"\n当前时间所有候选的验证码:")
        for i, secret in enumerate(candidates[:10]):  # 只显示前10个
            try:
                totp = pyotp.TOTP(secret)
                current_code = totp.now()
                print(f"{i+1:2d}. {current_code} - {secret[:25]}...")
            except:
                continue

    return found_matches

if __name__ == "__main__":
    results = check_all_candidates_for_380427()

    if results:
        print(f"\n🎉 可以使用的Twitter 2FA Secret:")
        for result in results:
            print(f"  {result['secret']}")
    else:
        print("\n需要重新获取正确的2FA secret或验证码")