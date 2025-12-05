#!/usr/bin/env python3
"""
方程式频道信号解析器
解析 "方程式-OI&Price异动（抓庄神器）" 频道的消息
"""

import re
from typing import Optional, Dict


def parse_fangchengshi_signal(text: str) -> Optional[Dict]:
    """
    解析方程式频道消息

    Returns:
        {
            'symbol': 'BTCUSDT',
            'oi_change_pct': 15.4,
            'price_change_pct': 8.3,
            'oi_value': 9.2,  # 单位: M USD
            'oi_marketcap_ratio': 24.3 or None,
            'price_24h_change': 26.5,
            'marketcap': 50.0 or None,  # 单位: M USD
            'direction': 'LONG' or 'SHORT' or 'UNKNOWN'
        }
    """
    if not text:
        return None

    # 尝试匹配币种 (支持特殊币种如 1000LUNCUSDT)
    symbol_pattern = r'([A-Z0-9]+USDT)'
    symbol_match = re.search(symbol_pattern, text)
    if not symbol_match:
        return None

    symbol = symbol_match.group(1)

    # 提取 OI 变化率
    oi_change_pattern = r'(?:持仓量|openinterest)\s*([+-]?\d+\.?\d*)%'
    oi_change_match = re.search(oi_change_pattern, text, re.IGNORECASE)
    if not oi_change_match:
        return None

    oi_change_pct = float(oi_change_match.group(1))

    # 提取价格变化 (过去 3600 秒)
    price_change_pattern = r'(?:价格|Price)\s*([+-]?\d+\.?\d*)%\s*in the past|过去\s*\d+\s*秒.*?([+-]?\d+\.?\d*)%'
    price_change_match = re.search(price_change_pattern, text, re.IGNORECASE)
    if not price_change_match:
        return None

    price_change_pct = float(price_change_match.group(1) or price_change_match.group(2))

    # 提取当前 OI 值
    oi_value_pattern = r'(?:持仓量|OI)[:：]\s*\$?(\d+\.?\d*)\s*[万M]'
    oi_value_match = re.search(oi_value_pattern, text)
    oi_value = None
    if oi_value_match:
        oi_value = float(oi_value_match.group(1))
        # 如果是"万美元"单位,转换为 M
        if '万' in text[oi_value_match.start():oi_value_match.end()+10]:
            oi_value = oi_value / 100  # 1万 = 0.01M

    # 提取 OI/市值比
    oi_marketcap_ratio = None
    ratio_pattern = r'(?:持仓量/市值比|OI/Marketcap ratio)[:：]\s*(\d+\.?\d*)%|不适用|N/A'
    ratio_match = re.search(ratio_pattern, text, re.IGNORECASE)
    if ratio_match and ratio_match.group(1):
        oi_marketcap_ratio = float(ratio_match.group(1))

    # 提取 24h 价格变化
    price_24h_pattern = r'24.*?(?:价格变化|Price Change)[:：]\s*([+-]?\d+\.?\d*)%'
    price_24h_match = re.search(price_24h_pattern, text, re.IGNORECASE)
    price_24h_change = None
    if price_24h_match:
        price_24h_change = float(price_24h_match.group(1))

    # 提取市值 (可选)
    marketcap = None
    marketcap_pattern = r'MarketCap[:：]\s*\$(\d+)M'
    marketcap_match = re.search(marketcap_pattern, text)
    if marketcap_match:
        marketcap = float(marketcap_match.group(1))

    # 判断多空方向
    direction = 'UNKNOWN'
    if oi_change_pct > 0 and price_change_pct > 0:
        direction = 'LONG'  # OI↑ + Price↑ → 做多力量
    elif oi_change_pct > 0 and price_change_pct < 0:
        direction = 'SHORT'  # OI↑ + Price↓ → 做空力量
    elif oi_change_pct < 0 and price_change_pct < 0:
        direction = 'CLOSE'  # OI↓ + Price↓ → 平仓/止损

    return {
        'symbol': symbol,
        'oi_change_pct': oi_change_pct,
        'price_change_pct': price_change_pct,
        'oi_value': oi_value,
        'oi_marketcap_ratio': oi_marketcap_ratio,
        'price_24h_change': price_24h_change,
        'marketcap': marketcap,
        'direction': direction
    }


def format_fangchengshi_signal(data: Dict) -> str:
    """
    将解析结果转换为 Valuescan 风格的消息
    """
    symbol = data['symbol']
    oi_change = data['oi_change_pct']
    price_change = data['price_change_pct']
    oi_value = data.get('oi_value', 0)
    ratio = data.get('oi_marketcap_ratio')
    price_24h = data.get('price_24h_change')
    marketcap = data.get('marketcap')
    direction = data['direction']

    # 方向emoji和文字
    if direction == 'LONG':
        emoji = '📈'
        direction_text = '资金流入(做多)'
        analysis = '持仓量和价格同步上涨，表明多头力量强劲，主力正在积极做多，建议关注做多机会。'
    elif direction == 'SHORT':
        direction_text = '资金流入(做空)'
        emoji = '📉'
        analysis = '持仓量上涨但价格下跌，表明空头力量强，主力正在做空，需警惕继续下跌风险。'
    elif direction == 'CLOSE':
        emoji = '⚠️'
        direction_text = '平仓/止损'
        analysis = '持仓量和价格同步下跌，表明主力正在平仓或止损离场，市场情绪转弱。'
    else:
        emoji = '❓'
        direction_text = '信号不明'
        analysis = '持仓量和价格走势不一致，建议谨慎观望。'

    # 构造消息
    message = f"""🔥 方程式OI&Price异动预警

{emoji} {direction_text}: {symbol}
📊 OI变化: {oi_change:+.1f}%
💹 价格变化(1h): {price_change:+.1f}%
💰 当前OI: ${oi_value:.1f}M"""

    if ratio is not None:
        message += f"\n📈 OI/市值比: {ratio:.1f}%"

    if price_24h is not None:
        message += f"\n📅 24h涨跌: {price_24h:+.1f}%"

    if marketcap is not None:
        message += f"\n💎 市值: ${marketcap:.0f}M"

    message += f"""

[双重异动分析]
{analysis}
数据来源: 方程式-OI&Price异动（抓庄神器）
"""

    return message


# 测试代码
if __name__ == "__main__":
    # 测试用例1: 做多信号
    test_msg1 = """🇨🇳 1000LUNCUSDT 币安持仓量增加15.4%，过去3600秒价格上涨8.3%，持仓量：920万美元，持仓量/市值比：不适用，24小时价格变化：+26.5%
🇺🇸 1000LUNCUSDT Binance openinterest +15.4%, Price +8.3% in the past 3600 seconds, OI: $9.2M, OI/Marketcap ratio: N/A, 24H Price Change: +26.5%"""

    # 测试用例2: 做空信号
    test_msg2 = """🇨🇳 SKYAIUSDT 币安未平仓合约在过去3600秒内减少27.6%，价格下跌28.0%，未平仓合约量：570万美元，未平仓合约/市值比率：16.6%，24小时价格变化：-3.6%
🇺🇸 SKYAIUSDT Binance openinterest -27.6%, Price -28.0% in the past 3600 seconds, OI: $5.7M, OI/Marketcap ratio: 16.6%, 24H Price Change: -3.6%

💰 市值
$SKYAI  MarketCap: $38M"""

    print("测试用例 1 (做多信号):")
    print("=" * 60)
    result1 = parse_fangchengshi_signal(test_msg1)
    if result1:
        print("解析结果:", result1)
        print("\n转换后的消息:")
        print(format_fangchengshi_signal(result1))
    else:
        print("❌ 解析失败")

    print("\n\n测试用例 2 (平仓信号):")
    print("=" * 60)
    result2 = parse_fangchengshi_signal(test_msg2)
    if result2:
        print("解析结果:", result2)
        print("\n转换后的消息:")
        print(format_fangchengshi_signal(result2))
    else:
        print("❌ 解析失败")
