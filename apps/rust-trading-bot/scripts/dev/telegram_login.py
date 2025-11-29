#!/usr/bin/env python3
"""
Telegram 登录辅助工具
自动处理验证码输入
"""

import subprocess
import sys
import time

def main():
    if len(sys.argv) < 2:
        print("用法: ./telegram_login.py <验证码>")
        print("例如: ./telegram_login.py 12345")
        sys.exit(1)

    code = sys.argv[1].strip()

    print(f"📱 使用验证码: {code}")
    print("🔄 启动 Telegram 客户端...\n")

    # 启动子进程
    process = subprocess.Popen(
        ['./target/release/get_channels'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # 实时读取输出
    try:
        while True:
            line = process.stdout.readline()
            if not line:
                break
            print(line, end='')

            # 检测到验证码提示时，自动输入
            if '请输入收到的验证码' in line:
                print(f"\n✅ 自动输入验证码: {code}")
                process.stdin.write(code + '\n')
                process.stdin.flush()
                time.sleep(0.5)

            # 检测到密码提示时
            if '需要两步验证密码' in line:
                print("\n⚠️  需要两步验证密码，请手动输入：")
                password = input()
                process.stdin.write(password + '\n')
                process.stdin.flush()

        # 读取剩余输出
        remaining = process.stdout.read()
        if remaining:
            print(remaining, end='')

        process.wait()

        if process.returncode != 0:
            stderr = process.stderr.read()
            if stderr:
                print(f"\n❌ 错误: {stderr}", file=sys.stderr)
            sys.exit(process.returncode)

    except KeyboardInterrupt:
        print("\n\n⚠️  用户中断")
        process.terminate()
        sys.exit(1)

if __name__ == '__main__':
    main()
