use crate::signal_manager::{CoinSignal, SignalPriority, SignalSource};
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    /// 币种符号匹配正则（匹配 BTC, ETH, SOL 等）
    static ref SYMBOL_REGEX: Regex = Regex::new(
        r"(?i)\b([A-Z]{2,10})(?:/|-)?(USDT?|USD|PERP)?\b"
    ).unwrap();

    /// 价格匹配正则（匹配 $1.23, 1.23U 等）
    static ref PRICE_REGEX: Regex = Regex::new(
        r"(?i)(?:\$|价格[:：\s]?)?(\d+(?:\.\d+)?)\s*(?:U|USDT|USD)?"
    ).unwrap();

    /// 方向匹配关键词
    static ref DIRECTION_KEYWORDS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("做多", "LONG");
        m.insert("多单", "LONG");
        m.insert("买入", "LONG");
        m.insert("开多", "LONG");
        m.insert("做空", "SHORT");
        m.insert("空单", "SHORT");
        m.insert("卖出", "SHORT");
        m.insert("开空", "SHORT");
        m.insert("平仓", "CLOSE");
        m.insert("止损", "STOP");
        m
    };

    /// 支持的币种白名单（主流币种）
    static ref WHITELIST: Vec<&'static str> = vec![
        "BTC", "ETH", "BNB", "SOL", "XRP", "ADA", "DOGE", "MATIC", "DOT",
        "LTC", "SHIB", "AVAX", "LINK", "UNI", "ATOM", "ETC", "XLM", "FIL",
        "APT", "ARB", "OP", "NEAR", "ICP", "INJ", "TIA", "SUI", "SEI",
        "PEPE", "WIF", "BONK", "FLOKI", "ORDI", "SATS", "JUP", "WLD",
        // === Binance Alpha 新币种 ===
        "XPL",
    ];
}

/// 解析结果
#[derive(Debug, Clone)]
pub struct ParsedCoinInfo {
    pub symbol: String,
    pub trading_pair: String,      // 如 "BTCUSDT"
    pub direction: Option<String>, // LONG/SHORT/CLOSE
    pub price: Option<f64>,
    pub confidence: f32, // 0.0-1.0
}

/// 币种解析器
pub struct CoinParser {
    strict_mode: bool, // 严格模式：只接受白名单币种
}

impl CoinParser {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// 从文本中解析币种信息
    pub fn parse(&self, text: &str) -> Vec<ParsedCoinInfo> {
        let mut results = Vec::new();

        // 提取所有可能的币种符号
        for cap in SYMBOL_REGEX.captures_iter(text) {
            if let Some(symbol_match) = cap.get(1) {
                let symbol = symbol_match.as_str().to_uppercase();

                // 白名单过滤
                if self.strict_mode && !WHITELIST.contains(&symbol.as_str()) {
                    debug!(
                        "⚠️ 严格模式: 忽略非白名单币种 '{}' | 原始文本前100字: {} | 建议: 添加到白名单或关闭严格模式(PARSE_STRICT_MODE=false)",
                        symbol,
                        text.chars().take(100).collect::<String>()
                    );
                    continue;
                }

                // 过滤常见误匹配（如 USD, API 等）
                if self.is_false_positive(&symbol) {
                    continue;
                }

                let quote = cap
                    .get(2)
                    .map(|m| m.as_str().to_uppercase())
                    .unwrap_or_else(|| "USDT".to_string());

                let trading_pair = format!("{}{}", symbol, quote);

                // 提取价格
                let price = self.extract_price(text);

                // 提取方向
                let direction = self.extract_direction(text);

                // 计算置信度
                let confidence =
                    self.calculate_confidence(text, &symbol, price.is_some(), direction.is_some());

                debug!(
                    "解析币种: {} ({}) | 价格: {:?} | 方向: {:?} | 置信度: {:.2}",
                    symbol, trading_pair, price, direction, confidence
                );

                results.push(ParsedCoinInfo {
                    symbol,
                    trading_pair,
                    direction,
                    price,
                    confidence,
                });
            }
        }

        // 按置信度排序
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        results
    }

    /// 从文本创建 CoinSignal
    pub fn parse_to_signal(&self, text: &str, source: SignalSource) -> Vec<CoinSignal> {
        let parsed = self.parse(text);

        parsed
            .into_iter()
            .filter(|info| {
                let pass = info.confidence >= 0.5;
                if !pass {
                    debug!(
                        "⚠️ 置信度不足: {} ({}) = {:.2} < 0.5 | 价格:{:?} 方向:{:?}",
                        info.symbol, info.trading_pair, info.confidence, info.price, info.direction
                    );
                }
                pass
            })
            .map(|info| {
                let priority = if info.direction.as_deref() == Some("STOP") {
                    SignalPriority::Critical
                } else if info.direction.is_some() {
                    SignalPriority::High
                } else {
                    SignalPriority::Medium
                };

                let mut signal = CoinSignal::new(info.trading_pair.clone(), source.clone())
                    .with_priority(priority)
                    .with_raw_data(text.to_string());

                if let Some(dir) = info.direction {
                    signal = signal.with_metadata("direction".to_string(), dir);
                }
                if let Some(price) = info.price {
                    signal = signal.with_metadata("price".to_string(), price.to_string());
                }
                signal =
                    signal.with_metadata("confidence".to_string(), info.confidence.to_string());

                signal
            })
            .collect()
    }

    /// 提取价格
    fn extract_price(&self, text: &str) -> Option<f64> {
        PRICE_REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .filter(|&price| price > 0.0 && price < 1_000_000.0)
    }

    /// 提取方向
    fn extract_direction(&self, text: &str) -> Option<String> {
        for (keyword, direction) in DIRECTION_KEYWORDS.iter() {
            if text.contains(keyword) {
                return Some(direction.to_string());
            }
        }
        None
    }

    /// 计算置信度
    fn calculate_confidence(
        &self,
        text: &str,
        symbol: &str,
        has_price: bool,
        has_direction: bool,
    ) -> f32 {
        let mut confidence: f32 = 0.3; // 基础分

        // 币种在白名单中 +0.2
        if WHITELIST.contains(&symbol) {
            confidence += 0.2;
        }

        // 包含价格信息 +0.2
        if has_price {
            confidence += 0.2;
        }

        // 包含方向信息 +0.3
        if has_direction {
            confidence += 0.3;
        }

        // 包含关键交易词汇 +0.1
        let trading_keywords = ["开仓", "建仓", "入场", "止损", "止盈", "杠杆"];
        if trading_keywords.iter().any(|kw| text.contains(kw)) {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// 检查是否为误匹配
    fn is_false_positive(&self, symbol: &str) -> bool {
        let false_positives = [
            "USD", "API", "HTTP", "JSON", "HTML", "XML", "URL", "AI", "ID", "IP", "OK", "NO",
            "YES", "NEW", "OLD",
        ];

        false_positives.contains(&symbol)
    }

    /// 规范化交易对格式
    pub fn normalize_trading_pair(pair: &str) -> String {
        // 移除 / - 等分隔符，统一为 BTCUSDT 格式
        pair.to_uppercase()
            .replace("/", "")
            .replace("-", "")
            .replace("_", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_symbol() {
        let parser = CoinParser::new(true);
        let results = parser.parse("BTC 突破新高！");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol, "BTC");
        assert_eq!(results[0].trading_pair, "BTCUSDT");
    }

    #[test]
    fn test_parse_with_price() {
        let parser = CoinParser::new(true);
        let results = parser.parse("ETH 价格: $2000 做多");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol, "ETH");
        assert_eq!(results[0].price, Some(2000.0));
        assert_eq!(results[0].direction, Some("LONG".to_string()));
    }

    #[test]
    fn test_parse_multiple_coins() {
        let parser = CoinParser::new(true);
        let results = parser.parse("关注 BTC 和 ETH，SOL 也不错");

        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|r| r.symbol == "BTC"));
        assert!(results.iter().any(|r| r.symbol == "ETH"));
        assert!(results.iter().any(|r| r.symbol == "SOL"));
    }

    #[test]
    fn test_whitelist_filtering() {
        let parser = CoinParser::new(true);
        let results = parser.parse("FAKE 币种不在白名单");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_false_positive_filtering() {
        let parser = CoinParser::new(false);
        let results = parser.parse("调用 API 获取 JSON 数据");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_confidence_calculation() {
        let parser = CoinParser::new(true);
        let results = parser.parse("BTC/USDT 开多 价格 $50000 止损 $49000");

        assert_eq!(results.len(), 1);
        assert!(results[0].confidence > 0.8); // 应该是高置信度
    }

    #[test]
    fn test_normalize_trading_pair() {
        assert_eq!(CoinParser::normalize_trading_pair("BTC/USDT"), "BTCUSDT");
        assert_eq!(CoinParser::normalize_trading_pair("ETH-USDT"), "ETHUSDT");
        assert_eq!(CoinParser::normalize_trading_pair("sol_usdt"), "SOLUSDT");
    }
}
