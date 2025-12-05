"""
配置管理模块
从根目录的 .env 文件读取配置
"""
import os
from pathlib import Path
from dotenv import load_dotenv

# 加载根目录的 .env 文件
ROOT_DIR = Path(__file__).parent.parent.parent
env_path = ROOT_DIR / ".env"
load_dotenv(env_path)

# Telegram配置
TELEGRAM_API_ID = os.getenv("TELEGRAM_API_ID")
TELEGRAM_API_HASH = os.getenv("TELEGRAM_API_HASH")
TELEGRAM_PHONE = os.getenv("TELEGRAM_PHONE")

# 监控的频道列表 (从环境变量读取,逗号分隔)
TELEGRAM_CHANNELS = os.getenv("TELEGRAM_CHANNELS", "").split(",")
TELEGRAM_CHANNELS = [ch.strip() for ch in TELEGRAM_CHANNELS if ch.strip()]

# Rust交易引擎配置
RUST_ENGINE_URL = os.getenv("RUST_ENGINE_URL", "http://localhost:8081")
RUST_ENGINE_TIMEOUT = int(os.getenv("RUST_ENGINE_TIMEOUT", "5"))

# 日志配置
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()  # 确保大写
LOG_FILE = os.getenv("LOG_FILE", "telegram_monitor.log")

# 会话文件路径
SESSION_FILE = os.getenv("TELEGRAM_SESSION_FILE", "telegram_session")

# 健康检查配置
HEALTH_CHECK_INTERVAL = int(os.getenv("HEALTH_CHECK_INTERVAL", "60"))  # 秒
MAX_ERROR_COUNT = int(os.getenv("MAX_ERROR_COUNT", "10"))

# 信号处理配置
SIGNAL_DEDUP_WINDOW = int(os.getenv("SIGNAL_DEDUP_WINDOW", "300"))  # 5分钟去重窗口
MAX_QUEUE_SIZE = int(os.getenv("MAX_QUEUE_SIZE", "1000"))

# OI (Open Interest) 监控配置
ENABLE_OI_MONITOR = os.getenv("ENABLE_OI_MONITOR", "true").lower() == "true"
OI_THRESHOLD = float(os.getenv("OI_THRESHOLD", "8.0"))  # OI变化率阈值(%)
OI_SCAN_INTERVAL = int(os.getenv("OI_SCAN_INTERVAL", "5"))  # 扫描周期(分钟)
OI_CONCURRENCY = int(os.getenv("OI_CONCURRENCY", "20"))  # 并发请求数

def validate_config():
    """验证配置是否完整"""
    errors = []

    if not TELEGRAM_API_ID:
        errors.append("缺少 TELEGRAM_API_ID")
    if not TELEGRAM_API_HASH:
        errors.append("缺少 TELEGRAM_API_HASH")
    if not TELEGRAM_PHONE:
        errors.append("缺少 TELEGRAM_PHONE")
    if not TELEGRAM_CHANNELS:
        errors.append("缺少 TELEGRAM_CHANNELS (需要在.env中配置)")

    if errors:
        raise ValueError(f"配置错误: {', '.join(errors)}")

    return True

if __name__ == "__main__":
    print(f"配置文件路径: {env_path}")
    print(f"API ID: {TELEGRAM_API_ID}")
    print(f"API Hash: {TELEGRAM_API_HASH[:10]}...")
    print(f"监控频道数: {len(TELEGRAM_CHANNELS)}")
    print(f"Rust引擎: {RUST_ENGINE_URL}")
    validate_config()
    print("✅ 配置验证通过")
