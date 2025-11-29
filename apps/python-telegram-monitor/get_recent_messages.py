#!/usr/bin/env python3
"""获取最近的Telegram消息"""
import asyncio
from datetime import datetime, timedelta
from telethon import TelegramClient

# 导入统一配置
from config import (
    TELEGRAM_API_ID,
    TELEGRAM_API_HASH,
    TELEGRAM_PHONE,
    TELEGRAM_CHANNELS,
    SESSION_FILE
)

# TELEGRAM_CHANNELS 是列表，转为字符串用于显示
CHANNELS_STR = ','.join(TELEGRAM_CHANNELS) if isinstance(TELEGRAM_CHANNELS, list) else TELEGRAM_CHANNELS

async def get_recent_messages():
    """获取最近的消息"""
    client = TelegramClient('telegram_session', TELEGRAM_API_ID, TELEGRAM_API_HASH)

    try:
        await client.start(phone=TELEGRAM_PHONE)

        # 获取用户信息
        me = await client.get_me()
        print(f"✅ 已登录: {me.first_name} (ID: {me.id})\n")

        channels = TELEGRAM_CHANNELS.split(',')

        for channel_username in channels:
            channel_username = channel_username.strip()
            print(f"\n{'='*80}")
            print(f"📡 频道: {channel_username}")
            print('='*80)

            try:
                # 获取频道实体
                entity = await client.get_entity(channel_username)
                print(f"✅ 频道名称: {entity.title}")

                # 获取最近10条消息
                messages = []
                async for message in client.iter_messages(entity, limit=10):
                    if message.text:
                        messages.append(message)

                print(f"\n📬 最近 {len(messages)} 条消息:\n")

                for idx, msg in enumerate(messages, 1):
                    print(f"\n--- 消息 #{idx} ---")
                    print(f"⏰ 时间: {msg.date.strftime('%Y-%m-%d %H:%M:%S')}")
                    print(f"📝 内容:\n{msg.text}")
                    print(f"🔗 链接: https://t.me/{channel_username.lstrip('@')}/{msg.id}")
                    print("-" * 80)

            except Exception as e:
                print(f"❌ 获取频道 {channel_username} 失败: {e}")

    except Exception as e:
        print(f"❌ 连接失败: {e}")
    finally:
        await client.disconnect()

if __name__ == '__main__':
    asyncio.run(get_recent_messages())
