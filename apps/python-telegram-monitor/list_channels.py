#!/usr/bin/env python3
"""列出用户已加入的所有Telegram频道和群组"""

import asyncio
import sys
import os

# 添加项目根目录到Python路径
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from telethon import TelegramClient
from config import TELEGRAM_API_ID, TELEGRAM_API_HASH, TELEGRAM_PHONE

async def list_channels():
    """列出所有已加入的频道和群组"""
    # 使用已存在的session文件
    client = TelegramClient('telegram_session', TELEGRAM_API_ID, TELEGRAM_API_HASH)

    try:
        await client.connect()

        if not await client.is_user_authorized():
            print("❌ Telegram会话已过期,请重新登录")
            await client.start(phone=TELEGRAM_PHONE)

        me = await client.get_me()
        print(f"\n✅ 已登录: {me.first_name} (ID: {me.id})")

        print("\n" + "="*80)
        print("📡 你已加入的Telegram频道和群组列表")
        print("="*80 + "\n")

        channels = []
        groups = []
        chats = []

        async for dialog in client.iter_dialogs():
            entity = dialog.entity

            # 频道 (Channel)
            if hasattr(entity, 'broadcast') and entity.broadcast:
                username = f"@{entity.username}" if entity.username else "无用户名"
                channels.append({
                    'id': entity.id,
                    'title': entity.title,
                    'username': username,
                })

            # 超级群组 (Megagroup)
            elif hasattr(entity, 'megagroup') and entity.megagroup:
                username = f"@{entity.username}" if entity.username else "无用户名"
                groups.append({
                    'id': entity.id,
                    'title': entity.title,
                    'username': username,
                })

            # 普通群组/私聊
            else:
                chats.append({
                    'id': entity.id,
                    'title': getattr(entity, 'title', getattr(entity, 'first_name', '未知')),
                })

        # 打印频道
        if channels:
            print("📢 【频道 Channels】 (单向广播频道)")
            print("-" * 80)
            for i, ch in enumerate(channels, 1):
                print(f"{i:2d}. {ch['title']}")
                if ch['username'] != "无用户名":
                    print(f"    ✅ 用户名: {ch['username']} (可用于监控)")
                else:
                    print(f"    ⚠️  频道ID: {ch['id']} (使用ID: -{ch['id']})")
                print()
        else:
            print("⚠️  未加入任何频道\n")

        # 打印超级群组
        if groups:
            print("👥 【超级群组 Supergroups】")
            print("-" * 80)
            for i, gr in enumerate(groups, 1):
                print(f"{i:2d}. {gr['title']}")
                if gr['username'] != "无用户名":
                    print(f"    ✅ 用户名: {gr['username']} (可用于监控)")
                else:
                    print(f"    ⚠️  群组ID: {gr['id']} (使用ID: -{gr['id']})")
                print()
        else:
            print("⚠️  未加入任何超级群组\n")

        # 统计普通群组和私聊
        if chats:
            print(f"💬 【其他对话】: {len(chats)} 个 (普通群组/私聊,不支持监控)\n")

        print("="*80)
        print("💡 使用建议:")
        print("  1. 找到你想监控的频道/群组")
        print("  2. 如果有'用户名'(如 @valuescan),直接使用用户名")
        print("  3. 如果没有用户名,使用负数ID (如 -1001234567890)")
        print("  4. 在 .env 文件中配置:")
        print("     TELEGRAM_CHANNELS=@valuescan,@another_channel")
        print("  5. 多个频道用逗号分隔")
        print("="*80 + "\n")

    finally:
        await client.disconnect()

if __name__ == '__main__':
    asyncio.run(list_channels())
