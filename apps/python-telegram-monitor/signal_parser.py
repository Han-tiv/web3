"""
信号解析模块
复用Rust项目的信号解析逻辑
"""
import re
from dataclasses import dataclass
from typing import Optional
from enum import Enum

class SignalSide(Enum):
    LONG = "LONG"
    SHORT = "SHORT"

class SignalConfidence(Enum):
    HIGH = "HIGH"
    MEDIUM = "MEDIUM"
    LOW = "LOW"

@dataclass
class TradingSignal:
    """交易信号数据结构"""
    symbol: str
    side: SignalSide
    entry_price: float
    stop_loss: float
    take_profit: Optional[float] = None
    confidence: SignalConfidence = SignalConfidence.MEDIUM
    leverage: Optional[int] = None
    raw_message: str = ""
    timestamp: float = 0

def parse_signal(text: str) -> Optional[TradingSignal]:
    """
    解析Telegram消息中的交易信号

    支持格式示例:
    - BTCUSDT LONG 95000 SL:94000 TP:96000
    - BTC多 入场:95000 止损:94000
    - ETH/USDT 做多 @3500 止损3400
    """
    if not text:
        return None

    text = text.upper().strip()

    # 提取币种
    symbol_match = re.search(r'([A-Z]{2,10})(?:USDT|/USDT)?', text)
    if not symbol_match:
        return None

    base_symbol = symbol_match.group(1)
    matched_token = symbol_match.group(0)
    if matched_token.endswith("USDT"):
        symbol = matched_token.replace("/", "")
    else:
        symbol = f"{base_symbol}USDT"

    # 判断方向
    side = None
    if any(keyword in text for keyword in ['LONG', '做多', '多单', '买入', 'BUY']):
        side = SignalSide.LONG
    elif any(keyword in text for keyword in ['SHORT', '做空', '空单', '卖出', 'SELL']):
        side = SignalSide.SHORT

    if not side:
        return None

    # 提取入场价
    entry_patterns = [
        r'(?:入场|ENTRY|@|价格)[:\s]*([0-9]+\.?[0-9]*)',
        r'(?:^|\s)([0-9]+\.?[0-9]*)\s*(?:入场|ENTRY)',
        r'(?:^|\s)([0-9]+\.?[0-9]*)\s*(?:USDT|U)'
    ]

    entry_price = None
    for pattern in entry_patterns:
        match = re.search(pattern, text)
        if match:
            entry_price = float(match.group(1))
            break

    if not entry_price:
        return None

    # 提取止损价
    sl_patterns = [
        r'(?:止损|SL|STOP)[:\s]*([0-9]+\.?[0-9]*)',
        r'SL[:\s]*([0-9]+\.?[0-9]*)'
    ]

    stop_loss = None
    for pattern in sl_patterns:
        match = re.search(pattern, text)
        if match:
            stop_loss = float(match.group(1))
            break

    if not stop_loss:
        # 如果没有明确止损,使用默认2%止损
        if side == SignalSide.LONG:
            stop_loss = entry_price * 0.98
        else:
            stop_loss = entry_price * 1.02

    # 提取止盈价 (可选)
    tp_patterns = [
        r'(?:止盈|TP|TARGET)[:\s]*([0-9]+\.?[0-9]*)',
        r'TP[:\s]*([0-9]+\.?[0-9]*)'
    ]

    take_profit = None
    for pattern in tp_patterns:
        match = re.search(pattern, text)
        if match:
            take_profit = float(match.group(1))
            break

    # 提取杠杆 (可选)
    leverage = None
    leverage_match = re.search(r'([0-9]+)X', text)
    if leverage_match:
        leverage = int(leverage_match.group(1))

    # 判断置信度
    confidence = SignalConfidence.MEDIUM
    if any(keyword in text for keyword in ['强烈', 'STRONG', '高', 'HIGH', '推荐']):
        confidence = SignalConfidence.HIGH
    elif any(keyword in text for keyword in ['观望', 'WATCH', '低', 'LOW', '谨慎']):
        confidence = SignalConfidence.LOW

    return TradingSignal(
        symbol=symbol,
        side=side,
        entry_price=entry_price,
        stop_loss=stop_loss,
        take_profit=take_profit,
        confidence=confidence,
        leverage=leverage,
        raw_message=text
    )

# 测试用例
if __name__ == "__main__":
    test_messages = [
        "BTCUSDT LONG 95000 SL:94000 TP:96000",
        "ETH做多 入场:3500 止损:3400",
        "SOL/USDT 做空 @145.5 止损147 10X",
        "BNB 买入 600 SL:590",
    ]

    print("=== 信号解析测试 ===\n")
    for msg in test_messages:
        signal = parse_signal(msg)
        if signal:
            print(f"✅ 原始: {msg}")
            print(f"   解析: {signal.symbol} {signal.side.value} @ {signal.entry_price}")
            print(f"   止损: {signal.stop_loss} | 止盈: {signal.take_profit}")
            print(f"   杠杆: {signal.leverage}x | 置信度: {signal.confidence.value}\n")
        else:
            print(f"❌ 解析失败: {msg}\n")
