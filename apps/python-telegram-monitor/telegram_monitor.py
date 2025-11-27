"""
Telegram监控主程序
使用Telethon监听频道消息,解析交易信号并发送到Rust交易引擎
"""
import asyncio
import aiohttp
import time
import logging
import colorlog
from datetime import datetime
from collections import deque
from typing import Dict, Set
from telethon import TelegramClient, events
from telethon.errors import SessionPasswordNeededError

from config import (
    TELEGRAM_API_ID, TELEGRAM_API_HASH, TELEGRAM_PHONE,
    TELEGRAM_CHANNELS, RUST_ENGINE_URL, RUST_ENGINE_TIMEOUT,
    LOG_LEVEL, LOG_FILE, SESSION_FILE,
    SIGNAL_DEDUP_WINDOW, MAX_QUEUE_SIZE, validate_config
)
# from signal_parser import parse_signal, TradingSignal  # 不再使用,直接透传原始消息给Rust

# 配置日志
def setup_logger():
    """设置彩色日志"""
    handler = colorlog.StreamHandler()
    handler.setFormatter(colorlog.ColoredFormatter(
        '%(log_color)s%(asctime)s [%(levelname)s] %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S',
        log_colors={
            'DEBUG': 'cyan',
            'INFO': 'green',
            'WARNING': 'yellow',
            'ERROR': 'red',
            'CRITICAL': 'red,bg_white',
        }
    ))

    logger = colorlog.getLogger('telegram_monitor')
    logger.addHandler(handler)
    logger.setLevel(LOG_LEVEL)

    # 同时输出到文件
    file_handler = logging.FileHandler(LOG_FILE, encoding='utf-8')
    file_handler.setFormatter(logging.Formatter(
        '%(asctime)s [%(levelname)s] %(message)s'
    ))
    logger.addHandler(file_handler)

    return logger

logger = setup_logger()

class TelegramMonitor:
    """Telegram频道监控器"""

    def __init__(self):
        self.client: Optional[TelegramClient] = None
        self.session: Optional[aiohttp.ClientSession] = None
        self.signal_history: deque = deque(maxlen=MAX_QUEUE_SIZE)
        self.last_messages: Dict[str, float] = {}  # 消息去重: {message_hash: timestamp}
        self.stats = {
            'messages_received': 0,
            'messages_sent': 0,
            'errors': 0,
            'started_at': time.time()
        }

    async def init_telegram_client(self):
        """初始化Telegram客户端"""
        logger.info("🔌 正在连接Telegram...")

        self.client = TelegramClient(SESSION_FILE, TELEGRAM_API_ID, TELEGRAM_API_HASH)

        await self.client.start(phone=TELEGRAM_PHONE)

        if not await self.client.is_user_authorized():
            logger.warning("⚠️  需要验证码,请输入验证码:")
            await self.client.send_code_request(TELEGRAM_PHONE)
            try:
                await self.client.sign_in(TELEGRAM_PHONE, input('输入验证码: '))
            except SessionPasswordNeededError:
                await self.client.sign_in(password=input('输入两步验证密码: '))

        me = await self.client.get_me()
        logger.info(f"✅ Telegram连接成功: {me.first_name} (@{me.username})")

        return self.client

    async def init_http_session(self):
        """初始化HTTP会话"""
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=RUST_ENGINE_TIMEOUT)
        )
        logger.info(f"🌐 HTTP会话已初始化: {RUST_ENGINE_URL}")

    def is_duplicate_message(self, message_text: str) -> bool:
        """检查消息是否重复（基于消息内容哈希）"""
        import hashlib
        message_hash = hashlib.md5(message_text.encode()).hexdigest()
        now = time.time()

        # 清理过期的消息记录
        expired_keys = [k for k, t in self.last_messages.items() if now - t > SIGNAL_DEDUP_WINDOW]
        for k in expired_keys:
            del self.last_messages[k]

        # 检查是否重复
        if message_hash in self.last_messages:
            last_time = self.last_messages[message_hash]
            if now - last_time < SIGNAL_DEDUP_WINDOW:
                return True

        # 记录新消息
        self.last_messages[message_hash] = now
        return False

    async def send_raw_message_to_rust(self, message_text: str, timestamp: float) -> bool:
        """发送原始Telegram消息到Rust交易引擎（让Rust自己解析）"""
        try:
            payload = {
                "raw_message": message_text,
                "timestamp": timestamp,
                "source": "telegram_raw"
            }

            async with self.session.post(
                f"{RUST_ENGINE_URL}/api/telegram/raw",
                json=payload
            ) as resp:
                if resp.status == 200:
                    result = await resp.json()
                    logger.info(f"✅ 原始消息已发送到Rust（{len(message_text)}字符）")
                    logger.debug(f"   消息预览: {message_text[:100]}...")
                    self.stats['messages_sent'] += 1
                    return True
                else:
                    error_text = await resp.text()
                    logger.error(f"❌ Rust引擎返回错误 {resp.status}: {error_text[:200]}")
                    self.stats['errors'] += 1
                    return False

        except asyncio.TimeoutError:
            logger.error(f"⏱️  发送超时: {RUST_ENGINE_TIMEOUT}秒")
            self.stats['errors'] += 1
            return False
        except Exception as e:
            logger.error(f"❌ 发送失败: {e}")
            self.stats['errors'] += 1
            return False

    async def handle_message(self, event):
        """处理频道消息 - 直接透传原始消息给Rust解析"""
        try:
            self.stats['messages_received'] += 1

            message_text = event.message.text
            if not message_text:
                return

            # 简单过滤：跳过明显不是信号的消息（如纯文本、链接等）
            if len(message_text) < 20 or not any(keyword in message_text for keyword in ['$', '资金', 'Alpha', 'FOMO', '异动']):
                logger.debug(f"📭 跳过非信号消息: {message_text[:30]}...")
                return

            logger.info(f"📨 收到Telegram消息（{len(message_text)}字符）")
            logger.debug(f"   消息预览: {message_text[:100]}...")

            # 去重检查（基于消息内容哈希）
            if self.is_duplicate_message(message_text):
                logger.warning(f"⚠️  重复消息已跳过")
                return

            # 直接发送原始消息到Rust（让Rust的parse_fund_alert解析）
            timestamp = event.message.date.timestamp()
            success = await self.send_raw_message_to_rust(message_text, timestamp)

            # 记录到历史
            self.signal_history.append({
                'message': message_text[:100],
                'success': success,
                'timestamp': time.time()
            })

        except Exception as e:
            logger.error(f"❌ 处理消息时出错: {e}", exc_info=True)
            self.stats['errors'] += 1

    async def print_stats(self):
        """定期打印统计信息"""
        while True:
            await asyncio.sleep(300)  # 每5分钟

            uptime = time.time() - self.stats['started_at']
            uptime_hours = uptime / 3600

            logger.info("📊 运行统计:")
            logger.info(f"   运行时间: {uptime_hours:.1f}小时")
            logger.info(f"   收到消息: {self.stats['messages_received']}")
            logger.info(f"   成功转发: {self.stats['messages_sent']}")
            logger.info(f"   错误次数: {self.stats['errors']}")
            logger.info(f"   去重缓存: {len(self.last_messages)}条")

    async def run(self):
        """主运行函数"""
        try:
            # 验证配置
            validate_config()

            # 初始化
            await self.init_telegram_client()
            await self.init_http_session()

            # 注册消息处理器
            channel_ids = []
            for channel in TELEGRAM_CHANNELS:
                try:
                    entity = await self.client.get_entity(channel)
                    channel_ids.append(entity.id)
                    logger.info(f"✅ 监听频道: {entity.title} (ID: {entity.id})")
                except Exception as e:
                    logger.error(f"❌ 无法获取频道 {channel}: {e}")

            if not channel_ids:
                raise ValueError("没有有效的频道可监听!")

            # 注册事件处理器
            @self.client.on(events.NewMessage(chats=channel_ids))
            async def message_handler(event):
                await self.handle_message(event)

            # 启动统计任务
            asyncio.create_task(self.print_stats())

            logger.info("🚀 Telegram监控已启动,等待消息...")
            logger.info(f"🎯 监控频道数: {len(channel_ids)}")
            logger.info(f"🔗 Rust引擎: {RUST_ENGINE_URL}")

            # 持续运行
            await self.client.run_until_disconnected()

        except KeyboardInterrupt:
            logger.info("⏹️  收到停止信号,正在关闭...")
        except Exception as e:
            logger.error(f"❌ 运行时错误: {e}", exc_info=True)
        finally:
            # 清理资源
            if self.session:
                await self.session.close()
            logger.info("👋 监控已停止")

async def main():
    """入口函数"""
    monitor = TelegramMonitor()
    await monitor.run()

if __name__ == "__main__":
    logger.info("=" * 60)
    logger.info("  Telegram交易信号监控 v1.0")
    logger.info("  Python监控 + Rust交易引擎 混合架构")
    logger.info("=" * 60)

    asyncio.run(main())
