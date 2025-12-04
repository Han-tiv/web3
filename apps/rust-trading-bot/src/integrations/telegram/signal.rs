// Telegram信号评分系统 - 移植自crypto-trading-bot (Go)
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Telegram信号评分记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSignal {
    pub id: Option<i64>,
    pub symbol: String,
    pub signal_type: String, // "强烈看多", "看多", "中性", "看空", "强烈看空"
    pub score: i32,          // -21 到 +10
    pub keywords: Vec<String>, // ["持续流入", "Alpha", ...]
    pub recommend_action: String, // "BUY", "SELL", "WATCH", "AVOID", "CLOSE/AVOID"
    pub reason: String,
    pub raw_message: String,
    pub timestamp: DateTime<Utc>,
}

impl TelegramSignal {
    /// 将keywords从Vec转为逗号分隔的字符串 (数据库存储)
    pub fn keywords_to_string(&self) -> String {
        self.keywords.join(", ")
    }

    /// 从逗号分隔的字符串恢复keywords (数据库读取)
    pub fn keywords_from_string(s: &str) -> Vec<String> {
        s.split(", ").map(|s| s.to_string()).collect()
    }
}

/// Telegram信号分析器
pub struct SignalAnalyzer {
    positive_keywords: HashMap<String, i32>,
    negative_keywords: HashMap<String, i32>,
    validity_duration: Duration,
}

impl SignalAnalyzer {
    pub fn new() -> Self {
        let mut positive_keywords = HashMap::new();
        positive_keywords.insert("持续流入".to_string(), 3);
        positive_keywords.insert("alpha".to_string(), 3);
        positive_keywords.insert("fomo".to_string(), 2);
        positive_keywords.insert("突破".to_string(), 2);
        positive_keywords.insert("强势".to_string(), 2);
        positive_keywords.insert("资金异动".to_string(), 1);
        positive_keywords.insert("24h内异动".to_string(), 1);
        positive_keywords.insert("放量".to_string(), 1);

        let mut negative_keywords = HashMap::new();
        negative_keywords.insert("主力资金已出逃".to_string(), -5);
        negative_keywords.insert("出逃".to_string(), -5);
        negative_keywords.insert("资金撤离".to_string(), -4);
        negative_keywords.insert("观望为主".to_string(), -3);
        negative_keywords.insert("注意市场风险".to_string(), -3);
        negative_keywords.insert("风险".to_string(), -2);
        negative_keywords.insert("及时止盈".to_string(), -2);
        negative_keywords.insert("止损".to_string(), -2);
        negative_keywords.insert("24h外异动".to_string(), -1);

        Self {
            positive_keywords,
            negative_keywords,
            validity_duration: Duration::hours(3), // 信号有效期3小时
        }
    }

    /// 分析单条消息并生成信号评分
    pub fn analyze_message(&self, symbol: String, text: &str) -> Option<TelegramSignal> {
        let text_lower = text.to_lowercase();
        let mut score = 0;
        let mut keywords = Vec::new();

        // 检查积极关键词
        for (keyword, points) in &self.positive_keywords {
            if text_lower.contains(&keyword.to_lowercase()) {
                score += points;
                keywords.push(format!("+{}", keyword));
            }
        }

        // 检查消极关键词
        for (keyword, points) in &self.negative_keywords {
            if text_lower.contains(&keyword.to_lowercase()) {
                score += points; // points已经是负数
                keywords.push(format!("-{}", keyword));
            }
        }

        // 根据评分确定信号类型和建议
        let (signal_type, recommend_action, reason) = if score >= 5 {
            ("强烈看多", "BUY", "多个积极信号叠加")
        } else if score >= 3 {
            ("看多", "BUY", "积极信号")
        } else if score >= 1 {
            ("中性偏多", "WATCH", "轻微积极信号")
        } else if score == 0 {
            ("中性", "WATCH", "无明显信号")
        } else if score >= -2 {
            ("中性偏空", "WATCH", "轻微风险信号")
        } else if score >= -4 {
            ("看空", "AVOID", "风险信号")
        } else {
            ("强烈看空", "CLOSE/AVOID", "严重风险警告")
        };

        Some(TelegramSignal {
            id: None,
            symbol,
            signal_type: signal_type.to_string(),
            score,
            keywords,
            recommend_action: recommend_action.to_string(),
            reason: reason.to_string(),
            raw_message: text.to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 检查信号是否在有效期内
    pub fn is_valid(&self, signal: &TelegramSignal) -> bool {
        let now = Utc::now();
        now.signed_duration_since(signal.timestamp) <= self.validity_duration
    }

    /// 格式化信号供前端展示
    pub fn format_signals(&self, signals: &[TelegramSignal]) -> String {
        let mut lines = vec![
            "**Telegram 市场信号 (最近3小时)**：".to_string(),
            "".to_string(),
        ];

        for signal in signals {
            let emoji = match signal.score {
                s if s >= 5 => "🔥🔥",
                s if s >= 3 => "📈",
                s if s >= 0 => "➡️",
                s if s >= -2 => "📉",
                s if s >= -4 => "📉",
                _ => "🚨",
            };

            lines.push(format!(
                "{} **{}**: {} (评分: {:+})",
                emoji, signal.symbol, signal.signal_type, signal.score
            ));
            lines.push(format!("   - 建议: {}", signal.recommend_action));
            lines.push(format!("   - 理由: {}", signal.reason));
            lines.push(format!("   - 关键词: {}", signal.keywords.join(", ")));
            lines.push("".to_string());
        }

        lines.push("**信号解读说明**：".to_string());
        lines.push("- 评分 ≥5: 强烈看多，可考虑入场".to_string());
        lines.push("- 评分 3-4: 看多，适度参与".to_string());
        lines.push("- 评分 1-2: 中性偏多，观察为主".to_string());
        lines.push("- 评分 -2~0: 中性或偏空，谨慎".to_string());
        lines.push("- 评分 ≤-3: 看空或风险警告，规避或平仓".to_string());

        lines.join("\n")
    }
}

impl Default for SignalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
