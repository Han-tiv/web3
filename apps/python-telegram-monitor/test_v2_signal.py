#!/usr/bin/env python3
"""模拟发送一个测试信号到Telegram频道,测试V2系统响应"""

import asyncio
import os
from telethon import TelegramClient
from dotenv import load_dotenv

# 加载根目录的 .env
load_dotenv('/home/hanins/code/web3/.env')

TELEGRAM_API_ID = os.getenv('TELEGRAM_API_ID')
TELEGRAM_API_HASH = os.getenv('TELEGRAM_API_HASH')
TELEGRAM_PHONE = os.getenv('TELEGRAM_PHONE')

# 测试信号 - 模拟一个AVNT信号(从真实频道消息修改)
TEST_SIGNAL = """🚨 **【Alpha + FOMO】****$AVNT**  🔥 **币安Alpha**
━━━━━━━━━
🔥 **检测到 Alpha + FOMO 信号！**
⚡ 在2小时内同时出现 Alpha 和 FOMO 信号

💵 当前价格: **$0.4311**
⭐ Alpha 信号: **1** 条
🚀 FOMO 信号: **1** 条

💡 操作建议:
   • 🎯 **高概率入场机会**
   • 📊 Alpha（价值机会）+ FOMO（市场情绪）
   • ✅ 可考虑适当参与
   • ⚠️ 注意控制仓位和风险
   • 🎯 及时设置止盈止损位

#Alpha + FOMO
━━━━━━━━━"""

async def send_test_signal():
    """发送测试信号"""
    client = TelegramClient('telegram_session', TELEGRAM_API_ID, TELEGRAM_API_HASH)

    try:
        await client.start(phone=TELEGRAM_PHONE)

        # 注意: 这个脚本仅用于测试系统是否能正常解析信号
        # 实际上我们不会发送消息到公共频道

        print("✅ 已连接到Telegram")
        print("\n📋 准备测试的信号:")
        print("="*80)
        print(TEST_SIGNAL)
        print("="*80)

        print("\n⚠️  注意: 此脚本不会实际发送消息到频道")
        print("   - 频道是单向广播,普通用户无法发送消息")
        print("   - 系统会自动监控频道发布的真实信号")
        print("   - 当频道发布新信号时,V2系统会自动分析")

        print("\n💡 测试建议:")
        print("   1. 等待频道发布新的交易信号")
        print("   2. 实时监控日志: tail -f trader.log")
        print("   3. 查看V2评分: grep 'V2评分' trader.log")

    except Exception as e:
        print(f"❌ 错误: {e}")
    finally:
        await client.disconnect()

if __name__ == '__main__':
    asyncio.run(send_test_signal())
