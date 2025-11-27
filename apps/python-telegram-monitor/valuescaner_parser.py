"""
Valuescaner频道信号解析器
专门解析valuescaner频道的特殊消息格式
"""
import re
from typing import Optional, Dict, Any


def parse_valuescaner_signal(text: str) -> Optional[Dict[str, Any]]:
    """
    解析valuescaner频道的信号消息

    支持的消息类型:
    1. 资金流入/流出: "📊 资金流入: PUMP 💰"
    2. FOMO信号: "🚀 【FOMO】$TRUST"
    3. Alpha信号: "⭐ 【Alpha】$AVNT"
    4. Alpha+FOMO: "🚨 【Alpha + FOMO】$AVNT"
    5. 资金异动: "💰 【资金异动】$PENGU"
    6. 主力资金出逃: "🚨 $SOL 主力资金已出逃"
    7. 价格高点警示: "📍 $NXPC 价格高点警示"
    8. 本金保护警示: "🟠 $NMR 本金保护警示"
    """
    if not text:
        return None

    # 提取币种符号
    symbol = None

    # 尝试多种格式提取币种
    patterns = [
        r'\$([A-Z]{2,10})',  # $BTC格式
        r'\*\*\$([A-Z]{2,10})\*\*',  # **$BTC**格式
        r'资金流入:\s*([A-Z]{2,10})',  # 资金流入: PUMP
        r'资金流出:\s*([A-Z]{2,10})',  # 资金流出: PUMP
    ]

    for pattern in patterns:
        match = re.search(pattern, text)
        if match:
            symbol = match.group(1)
            break

    if not symbol:
        return None

    # 标准化为USDT交易对
    if not symbol.endswith('USDT'):
        symbol = f"{symbol}USDT"

    # 提取当前价格
    price = None
    price_patterns = [
        r'现价[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',  # 现价: **$0.4311**
        r'💵\s*现价[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',
        r'价格[:\s]*\*\*\$([0-9]+\.?[0-9]*)\*\*',
    ]

    for pattern in price_patterns:
        match = re.search(pattern, text)
        if match:
            price = float(match.group(1))
            break

    # 提取24H涨跌幅
    change_24h = None
    change_patterns = [
        r'24H[:\s]*`([+-]?[0-9]+\.?[0-9]*)%`',
        r'📈\s*24H[:\s]*`([+-]?[0-9]+\.?[0-9]*)%`',
        r'📉\s*24H[:\s]*`([+-]?[0-9]+\.?[0-9]*)%`',
    ]

    for pattern in change_patterns:
        match = re.search(pattern, text)
        if match:
            change_24h = float(match.group(1))
            break

    # 判断信号类型和评分
    signal_type = None
    score = 0
    confidence = "LOW"
    risk_level = "NORMAL"

    text_upper = text.upper()

    if "主力资金已出逃" in text:
        signal_type = "fund_escape"
        score = -5
        confidence = "LOW"
        risk_level = "HIGH"
        should_long = False
    elif "资金流出" in text:
        signal_type = "fund_outflow"
        score = -3
        confidence = "LOW"
        risk_level = "MEDIUM"
        should_long = False
    elif "价格高点警示" in text:
        signal_type = "price_high_alert"
        score = -2
        confidence = "LOW"
        risk_level = "MEDIUM"
        should_long = False
    elif "本金保护警示" in text:
        signal_type = "capital_protection"
        score = -2
        confidence = "LOW"
        risk_level = "MEDIUM"
        should_long = False
    elif "ALPHA + FOMO" in text_upper or "ALPHA+FOMO" in text_upper:
        signal_type = "alpha_fomo"
        score = 7  # 高评分
        confidence = "HIGH"
        should_long = True
    elif "【FOMO】" in text_upper or "FOMO" in text_upper:
        signal_type = "fomo"
        score = 5
        confidence = "MEDIUM"
        should_long = True
    elif "【ALPHA】" in text_upper or "ALPHA" in text_upper:
        signal_type = "alpha"
        score = 5
        confidence = "MEDIUM"
        should_long = True
    elif "资金流入" in text:
        signal_type = "fund_inflow"
        score = 2
        confidence = "MEDIUM"
        should_long = True
    elif "【资金异动】" in text:
        signal_type = "fund_movement"
        score = 3
        confidence = "MEDIUM"
        should_long = True
    else:
        # 不是可识别的信号类型
        return None

    return {
        'symbol': symbol,
        'signal_type': signal_type,
        'score': score,
        'confidence': confidence,
        'price': price,
        'change_24h': change_24h,
        'should_long': should_long,
        'risk_level': risk_level,
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
            print(f"   类型: {signal['signal_type']} | 评分: {signal['score']} | 置信度: {signal['confidence']}")
            print(f"   价格: ${signal['price']} | 24H: {signal['change_24h']}%")
            print(f"   建议: {'做多' if signal['should_long'] else '观望/做空'}")
            print()
        else:
            print(f"❌ 解析失败")
            print()
