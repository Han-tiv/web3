"""
Valuescaner频道信号解析器
负责提取币种、价格及风控标记
"""
import re
from typing import Optional, Dict, Any

RISK_KEYWORDS = [
    "主力资金已出逃",
    "资金流出",
    "价格高点警示",
    "本金保护警示"
]

POSITIVE_KEYWORDS = [
    "【ALPHA + FOMO】",
    "ALPHA+FOMO",
    "【ALPHA】",
    "ALPHA",
    "【FOMO】",
    "FOMO",
    "资金流入",
    "【资金异动】"
]


def parse_valuescaner_signal(text: str) -> Optional[Dict[str, Any]]:
    """
    解析valuescaner频道的信号消息，仅提取币种、价格与是否可做多
    """
    if not text:
        return None

    # 提取币种符号
    symbol = None
    patterns = [
        r'\$([A-Z]{2,10})',  # $BTC格式
        r'\*\*\$([A-Z]{2,10})\*\*',  # **$BTC**格式
        r'资金流入:\s*([A-Z]{2,10})',  # 资金流入: PUMP
        r'资金流出:\s*([A-Z]{2,10})',  # 资金流出: PUMP
    ]

    for pattern in patterns:
        match = re.search(pattern, text, re.IGNORECASE)
        if match:
            symbol = match.group(1).upper()
            break

    if not symbol:
        return None

    if not symbol.endswith('USDT'):
        symbol = f"{symbol}USDT"

    # 提取当前价格（仅用于日志）
    price = None
    price_patterns = [
        r'现价[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',
        r'💵\s*现价[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',
        r'价格[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',
    ]

    for pattern in price_patterns:
        match = re.search(pattern, text)
        if match:
            price = float(match.group(1))
            break

    # 默认只要命中风险关键词则禁止做多，否则只有在正面关键词时才允许做多
    upper_text = text.upper()
    is_risky = any(keyword in text for keyword in RISK_KEYWORDS)
    has_positive = any(keyword.upper() in upper_text for keyword in POSITIVE_KEYWORDS)
    should_long = has_positive and not is_risky

    return {
        'symbol': symbol,
        'price': price,
        'should_long': should_long,
        'raw_text': text
    }


# 测试用例
if __name__ == "__main__":
    test_messages = [
        """🚨 **【Alpha + FOMO】****$AVNT**  🔥 **币安Alpha**
━━━━━━━━━
🔥 **检测到 Alpha + FOMO 信号！**
⚡ 在2小时内同时出现 Alpha 和 FOMO 信号

💵 当前价格: **$0.4311**
⭐ Alpha 信号: **1** 条
🚀 FOMO 信号: **1** 条""",

        """📊 资金流入: PUMP 💰
   价格: $0.0028 | 24H: -7.24% | 类型: 合约""",

        """⭐ **【Alpha】****$TRUST**** 🔥 币安Alpha**
━━━━━━━━━
💰 资金状态: 持续流入
💵 现价: **$0.2017**
📈 24H: `+95.42%`
📊 类型: 合约""",

        """🚨 **$SOL**** 主力资金已出逃**
━━━━━━━━━
⚠️ 资金异动实时追踪结束
💼 疑似主力资金已出逃，资金异动监控结束
💵 现价: **$128.58**
📉 24H跌幅: `-4.15%`""",
    ]

    print("=== Valuescaner信号解析测试 ===\n")
    for msg in test_messages:
        signal = parse_valuescaner_signal(msg)
        if signal:
            print(f"✅ 币种: {signal['symbol']}")
            print(f"   价格: ${signal['price']}")
            print(f"   建议: {'做多' if signal['should_long'] else '跳过'}")
            print()
        else:
            print("❌ 解析失败\n")
