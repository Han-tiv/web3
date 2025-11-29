#!/usr/bin/env python3
"""
Telegram信号转发器 - 使用稳定的Telethon库
接收Telegram消息并通过HTTP转发到Rust交易引擎
"""

import asyncio
import os
import sys
import time
from datetime import datetime
from typing import Optional
import re

import httpx
from telethon import TelegramClient, events

# 导入统一配置
from config import (
    TELEGRAM_API_ID,
    TELEGRAM_API_HASH,
    TELEGRAM_PHONE,
    TELEGRAM_CHANNELS,
    RUST_ENGINE_URL
)

# Rust交易引擎API地址
RUST_API_URL = f"{RUST_ENGINE_URL}/api/signals"

# 统计信息
stats = {
    'received': 0,
    'forwarded': 0,
    'skipped': 0,
    'failed': 0,
    'start_time': datetime.now()
}

# 币种提取与风险过滤规则
SYMBOL_PATTERNS = [
    re.compile(r'\$([A-Za-z0-9]{2,10})', re.IGNORECASE),                    # $BTC
    re.compile(r'资金(?:流入|流出)[:：\s]+([A-Za-z0-9]{2,10})', re.IGNORECASE),  # 资金流入: PUMP
    re.compile(r'\b([A-Za-z0-9]{2,10})/USDT\b', re.IGNORECASE),             # BTC/USDT
    re.compile(r'\b([A-Za-z0-9]{2,10})-USDT\b', re.IGNORECASE),             # BTC-USDT
    re.compile(r'\b([A-Za-z0-9]{2,10})USDT\b', re.IGNORECASE)               # BTCUSDT
]

RISK_PATTERNS = [
    re.compile(r'主力(?:资金)?(?:已)?出逃'),
    re.compile(r'资金流出'),
    re.compile(r'价格高点'),
    re.compile(r'本金保护')
]


def extract_symbol(text: str) -> Optional[str]:
    """直接基于Telegram原文提取币种并补全USDT"""
    if not text:
        return None

    for pattern in SYMBOL_PATTERNS:
        match = pattern.search(text)
        if not match:
            continue
        raw_symbol = match.group(1).upper()
        if raw_symbol.endswith('USDT'):
            return raw_symbol
        return f"{raw_symbol}USDT"
    return None


def is_risk_signal(text: str) -> bool:
    """风险关键词过滤（主力出逃/资金流出/价格高点/本金保护）"""
    if not text:
        return False
    return any(pattern.search(text) for pattern in RISK_PATTERNS)


class SignalForwarder:
    """信号转发器类"""

    def __init__(self):
        self.client = TelegramClient(
            'telegram_session',
            TELEGRAM_API_ID,
            TELEGRAM_API_HASH
        )
        self.http_client = httpx.AsyncClient(timeout=10.0)
        self.running = True

    async def start(self):
        """启动转发器"""
        try:
            # 启动Telegram客户端
            await self.client.start(phone=TELEGRAM_PHONE)

            # 获取用户信息
            me = await self.client.get_me()
            print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
            print(f"✅ Telethon已连接", flush=True)
            print(f"   用户: {me.first_name} (ID: {me.id})", flush=True)
            print(f"   监控频道: {', '.join(TELEGRAM_CHANNELS)}", flush=True)
            print(f"   转发目标: {RUST_API_URL}", flush=True)
            print(f"   解析器: 轻量正则解析", flush=True)
            print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
            print(f"📡 开始监控Telegram消息...", flush=True)
            print(flush=True)

            # 注册消息处理器
            @self.client.on(events.NewMessage(chats=TELEGRAM_CHANNELS))
            async def message_handler(event):
                await self.handle_message(event)

            # 定期输出统计信息
            asyncio.create_task(self.print_stats())

            # 运行直到断开连接
            await self.client.run_until_disconnected()

        except KeyboardInterrupt:
            print("\n⚠️  收到中断信号,正在关闭...")
        except Exception as e:
            print(f"❌ 启动失败: {e}")
            import traceback
            traceback.print_exc()
            sys.exit(1)
        finally:
            await self.cleanup()

    async def handle_message(self, event):
        """处理收到的消息"""
        try:
            stats['received'] += 1

            # 获取频道信息
            channel_username = event.chat.username if event.chat else 'unknown'
            message_text = event.text or ''

            # 输出接收日志
            timestamp = event.date.strftime('%H:%M:%S')
            print(f"📨 [{timestamp}] 消息 #{event.id} (来自 @{channel_username})")
            if len(message_text) > 80:
                print(f"   内容: {message_text[:80]}...")
            else:
                print(f"   内容: {message_text.split(chr(10))[0]}")  # 只显示第一行

            # 风险关键词过滤
            if is_risk_signal(message_text):
                stats['skipped'] += 1
                print(f"   ⏭️  风险信号,跳过")
                print()
                return

            symbol = extract_symbol(message_text)
            if not symbol:
                stats['skipped'] += 1
                print("   ⏭️  缺少币种信息,跳过")
                print()
                return

            signal_data = {
                'symbol': symbol,
                'raw_message': message_text,
                'timestamp': time.time()
            }

            print(f"   🎯 币种: {symbol}")
            print(f"      Payload字段: symbol/raw_message/timestamp")

            # 转发到Rust引擎
            try:
                response = await self.http_client.post(
                    RUST_API_URL,
                    json=signal_data,
                    timeout=10.0
                )

                if response.status_code == 200:
                    stats['forwarded'] += 1
                    print(f"   ✅ 已转发到Rust引擎")
                    try:
                        response_json = response.json()
                        msg = response_json.get('message', '')
                        if msg:
                            print(f"      响应: {msg}")
                    except:
                        pass
                else:
                    stats['failed'] += 1
                    print(f"   ⚠️  Rust引擎返回错误: {response.status_code}")
                    print(f"      响应: {response.text[:100]}")

            except httpx.ConnectError:
                stats['failed'] += 1
                print(f"   ❌ 连接Rust引擎失败")
                print(f"      地址: {RUST_API_URL}")
                print(f"      提示: 请确认Rust交易引擎正在运行")
            except httpx.TimeoutException:
                stats['failed'] += 1
                print(f"   ❌ 转发超时 (10秒)")
            except Exception as e:
                stats['failed'] += 1
                print(f"   ❌ 转发失败: {e}")

            print()  # 空行分隔

        except Exception as e:
            print(f"❌ 处理消息失败: {e}")
            import traceback
            traceback.print_exc()
            print()

    async def print_stats(self):
        """定期输出统计信息"""
        while self.running:
            await asyncio.sleep(300)  # 每5分钟

            uptime = datetime.now() - stats['start_time']
            hours = uptime.total_seconds() / 3600

            print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            print(f"📊 运行统计 (运行时长: {hours:.1f}小时)")
            print(f"   接收消息: {stats['received']}")
            print(f"   成功转发: {stats['forwarded']}")
            print(f"   跳过消息: {stats['skipped']}")
            print(f"   失败次数: {stats['failed']}")
            if stats['received'] > 0:
                forward_rate = (stats['forwarded'] / stats['received']) * 100
                print(f"   转发率: {forward_rate:.1f}%")
            print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            print()

    async def cleanup(self):
        """清理资源"""
        self.running = False
        await self.http_client.aclose()
        await self.client.disconnect()
        print("✅ 资源已清理")


async def main():
    """主函数"""
    # 检查必要的环境变量
    if not TELEGRAM_API_ID or TELEGRAM_API_ID == 0:
        print("❌ 错误: TELEGRAM_API_ID 未配置")
        print("   请在 /home/hanins/code/web3/.env 中设置")
        sys.exit(1)

    if not TELEGRAM_API_HASH:
        print("❌ 错误: TELEGRAM_API_HASH 未配置")
        sys.exit(1)

    if not TELEGRAM_PHONE:
        print("❌ 错误: TELEGRAM_PHONE 未配置")
        sys.exit(1)

    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
    print("🚀 Telegram信号转发器启动", flush=True)
    print("   使用库: Telethon v1.42+ (Production/Stable)", flush=True)
    print("   架构: Python (Telegram) → HTTP → Rust (AI引擎)", flush=True)
    print("   频道: valuescaner", flush=True)
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", flush=True)
    print(flush=True)

    # 创建并启动转发器
    forwarder = SignalForwarder()
    await forwarder.start()


if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n✅ 程序已退出")
