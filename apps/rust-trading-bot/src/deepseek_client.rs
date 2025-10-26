use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use log::{info, warn};

#[derive(Debug, Serialize)]
pub struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
pub struct DeepSeekResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: MessageContent,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradingSignal {
    pub signal: String,      // "BUY", "SELL", "HOLD"
    pub reason: String,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub confidence: String,  // "HIGH", "MEDIUM", "LOW"
}

pub struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl DeepSeekClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
        }
    }

    /// 分析市场并生成交易信号
    pub async fn analyze_market(&self, prompt: &str) -> Result<TradingSignal> {
        let request = DeepSeekRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
            temperature: Some(0.7),
        };

        info!("🧠 调用 DeepSeek API...");
        
        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("DeepSeek API error ({}): {}", status, error_text);
        }

        let deepseek_response: DeepSeekResponse = response.json().await
            .context("Failed to parse DeepSeek response")?;
        
        info!("✅ DeepSeek 响应: {} tokens", deepseek_response.usage.total_tokens);
        
        // 解析 JSON 响应
        let content = &deepseek_response.choices[0].message.content;
        let signal: TradingSignal = serde_json::from_str(content)
            .context("Failed to parse trading signal from DeepSeek response")?;
        
        info!("📡 交易信号: {} | 置信度: {}", signal.signal, signal.confidence);
        
        Ok(signal)
    }

    /// 构建分析 prompt
    pub fn build_prompt(
        &self,
        klines: &[Kline],
        indicators: &TechnicalIndicators,
        sentiment: Option<&MarketSentiment>,
        position: Option<&Position>,
        current_price: f64,
    ) -> String {
        let kline_text = self.format_klines(klines);
        let indicator_text = self.format_indicators(indicators);
        let sentiment_text = sentiment.map(|s| self.format_sentiment(s))
            .unwrap_or_else(|| "【市场情绪】\n数据获取失败".to_string());
        let position_text = self.format_position(position);

        format!(
            r#"你是一个专业的加密货币交易分析师。请基于以下BTC/USDT 15m周期数据进行分析：

{}

{}

{}

【当前行情】
- 当前价格: ${:.2}
- 当前持仓: {}

【分析要求】
1. 基于15m K线趋势和技术指标给出交易信号: BUY(买入) / SELL(卖出) / HOLD(观望)
2. 简要分析理由（考虑趋势连续性、支撑阻力、成交量等因素）
3. 基于技术分析建议合理的止损价位
4. 基于技术分析建议合理的止盈价位
5. 评估信号信心程度

请用以下JSON格式回复：
{{
    "signal": "BUY|SELL|HOLD",
    "reason": "分析理由",
    "stop_loss": 具体价格,
    "take_profit": 具体价格,
    "confidence": "HIGH|MEDIUM|LOW"
}}
"#,
            kline_text,
            indicator_text,
            sentiment_text,
            current_price,
            if position.is_some() { "有持仓" } else { "无持仓" }
        )
    }

    fn format_klines(&self, klines: &[Kline]) -> String {
        let mut text = String::from("【最近5根15m K线数据】\n");
        
        let recent_klines: Vec<_> = klines.iter().rev().take(5).collect();
        for (i, kline) in recent_klines.iter().rev().enumerate() {
            let trend = if kline.close > kline.open { "阳线" } else { "阴线" };
            let change = ((kline.close - kline.open) / kline.open) * 100.0;
            
            text.push_str(&format!(
                "K线{}: {} 开盘:{:.2} 收盘:{:.2} 最高:{:.2} 最低:{:.2} 涨跌:{:+.2}%\n",
                i + 1, trend, kline.open, kline.close, kline.high, kline.low, change
            ));
        }
        
        text
    }

    fn format_indicators(&self, indicators: &TechnicalIndicators) -> String {
        format!(
            r#"【技术指标】
SMA 5: {:.2}
SMA 20: {:.2}
SMA 50: {:.2}
RSI: {:.2}
MACD: {:.4}
MACD Signal: {:.4}
布林带上轨: {:.2}
布林带中轨: {:.2}
布林带下轨: {:.2}"#,
            indicators.sma_5,
            indicators.sma_20,
            indicators.sma_50,
            indicators.rsi,
            indicators.macd,
            indicators.macd_signal,
            indicators.bb_upper,
            indicators.bb_middle,
            indicators.bb_lower
        )
    }

    fn format_sentiment(&self, sentiment: &MarketSentiment) -> String {
        format!(
            r#"【市场情绪】
恐慌贪婪指数: {} ({})
24小时价格变化: {:+.2}%
长短比: {:.2}"#,
            sentiment.fear_greed_value,
            sentiment.fear_greed_label,
            sentiment.price_change_24h,
            sentiment.long_short_ratio
        )
    }

    fn format_position(&self, position: Option<&Position>) -> String {
        match position {
            Some(pos) => format!(
                r#"{}仓, 数量: {:.4} BTC, 入场价: ${:.2}, 盈亏: ${:.2}"#,
                if pos.side == "long" { "多头" } else { "空头" },
                pos.size,
                pos.entry_price,
                pos.unrealized_pnl
            ),
            None => "无持仓".to_string(),
        }
    }
}

// 数据结构
#[derive(Debug, Clone)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TechnicalIndicators {
    pub sma_5: f64,
    pub sma_20: f64,
    pub sma_50: f64,
    pub rsi: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub bb_upper: f64,
    pub bb_middle: f64,
    pub bb_lower: f64,
}

#[derive(Debug, Clone)]
pub struct MarketSentiment {
    pub fear_greed_value: i32,
    pub fear_greed_label: String,
    pub price_change_24h: f64,
    pub long_short_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub side: String,
    pub size: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
}
