#!/usr/bin/env python3
"""
Telegram信号转发器 - 使用稳定的Telethon库
接收Telegram消息并通过HTTP转发到Rust交易引擎
专门为valuescaner频道优化
"""

import asyncio
import json
import os
import sys
import time
from datetime import datetime
from typing import Optional

import httpx
from dotenv import load_dotenv
from telethon import TelegramClient, events
from telethon.tl.types import User

# 导入valuescaner专用解析器
from valuescaner_parser import parse_valuescaner_signal

# 加载根目录的 .env
load_dotenv('/home/hanins/code/web3/.env')

# 配置
TELEGRAM_API_ID = int(os.getenv('TELEGRAM_API_ID', '0'))
TELEGRAM_API_HASH = os.getenv('TELEGRAM_API_HASH', '')
TELEGRAM_PHONE = os.getenv('TELEGRAM_PHONE', '')
TELEGRAM_CHANNELS = os.getenv('TELEGRAM_CHANNELS', '@valuescaner').split(',')

# Rust交易引擎API地址
RUST_API_URL = os.getenv('RUST_API_URL', 'http://localhost:8080/api/signals')

# 统计信息
stats = {
    'received': 0,
    'forwarded': 0,
    'skipped': 0,
    'failed': 0,
    'start_time': datetime.now()
}


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
            print(f"   解析器: Valuescaner专用", flush=True)
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

            # 解析valuescaner信号
            parsed = parse_valuescaner_signal(message_text)

            if not parsed:
                stats['skipped'] += 1
                print(f"   ⏭️  非交易信号,跳过")
                print()
                return

            # 输出解析结果
            print(f"   🎯 币种: {parsed['symbol']}")
            print(f"      类型: {parsed['signal_type']} | 评分: {parsed['score']} | 置信度: {parsed['confidence']}")
            if parsed['price']:
                print(f"      价格: ${parsed['price']:.4f}", end='')
                if parsed['change_24h'] is not None:
                    print(f" | 24H: {parsed['change_24h']:+.2f}%")
                else:
                    print()
            else:
                stats['skipped'] += 1
                print("   ⏭️  缺少价格信息, 跳过")
                print()
                return

            # 只转发应该做多的信号
            if not parsed['should_long']:
                stats['skipped'] += 1
                print(f"   ⏭️  风险信号,跳过 (signal_type={parsed['signal_type']})")
                print()
                return

            # 构建发送到Rust的数据 (匹配TelegramSignalPayload结构)
            price = parsed['price']

            signal_data = {
                'symbol': parsed['symbol'],
                'side': 'LONG',  # 所有转发的信号都是做多信号
                'entry_price': price,
                'stop_loss': price * 0.95,
                'take_profit': price * 1.10,
                'confidence': parsed['confidence'],  # "HIGH", "MEDIUM", "LOW"
                'leverage': 10,  # 默认10x杠杆
                'source': 'telegram_python',
                'timestamp': time.time(),
                'raw_message': message_text,
                'signal_type': parsed['signal_type'],
                'score': parsed['score'],
                'risk_level': parsed.get('risk_level', 'NORMAL')
            }

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
