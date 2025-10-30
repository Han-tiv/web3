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

    /// 构建分析 prompt (纯技术指标版本)
    pub fn build_prompt(
        &self,
        klines: &[Kline],
        indicators: &TechnicalIndicators,
        current_price: f64,
        position: Option<&Position>,
    ) -> String {
        let kline_text = self.format_klines(klines);
        let indicator_text = self.format_indicators(indicators);
        let position_text = self.format_position(position);
        
        // 趋势分析
        let trend_analysis = self.analyze_trend(indicators, current_price);
        
        format!(
            r#"你是一个专业的加密货币交易分析师。请基于以下BTC/USDT 15m周期数据进行分析：

{}

{}

【当前行情】
- 当前价格: ${:.2}
- 当前持仓: {}

【防频繁交易重要原则】
1. **趋势持续性优先**: 不要因单根K线或短期波动改变整体趋势判断
2. **持仓稳定性**: 除非趋势明确强烈反转，否则保持现有持仓方向
3. **反转确认**: 需要至少2-3个技术指标同时确认趋势反转才改变信号
4. **成本意识**: 减少不必要的仓位调整，每次交易都有成本

【交易指导原则 - 必须遵守】
1. **趋势跟随**: 明确趋势出现时立即行动，不要过度等待
2. **因为做的是BTC，做多权重可以大一点点**
3. **信号明确性**:
   - 强势上涨趋势 → BUY信号
   - 强势下跌趋势 → SELL信号
   - 仅在窄幅震荡、无明确方向时 → HOLD信号
4. **技术指标权重**:
   - 趋势(均线排列) > RSI > MACD > 布林带
   - 价格突破关键支撑/阻力位是重要信号

【当前技术状况分析】
{}

【智能仓位管理规则 - 必须遵守】
1. **减少过度保守**：
   - 明确趋势中不要因轻微超买/超卖而过度HOLD
   - RSI在30-70区间属于健康范围，不应作为主要HOLD理由
   - 布林带位置在20%-80%属于正常波动区间

2. **趋势跟随优先**：
   - 强势上涨趋势 + 任何RSI值 → 积极BUY信号
   - 强势下跌趋势 + 任何RSI值 → 积极SELL信号
   - 震荡整理 + 无明确方向 → HOLD信号

3. **突破交易信号**：
   - 价格突破关键阻力 + 成交量放大 → 高信心BUY
   - 价格跌破关键支撑 + 成交量放大 → 高信心SELL

4. **持仓优化逻辑**：
   - 已有持仓且趋势延续 → 保持或BUY/SELL信号
   - 趋势明确反转 → 及时反向信号
   - 不要因为已有持仓而过度HOLD

【重要】请基于技术分析做出明确判断，避免因过度谨慎而错过趋势行情！

【分析要求】
基于以上分析，请给出明确的交易信号。

请用以下JSON格式回复：
{{
    "signal": "BUY|SELL|HOLD",
    "reason": "简要分析理由(包含趋势判断和技术依据)",
    "stop_loss": 具体价格,
    "take_profit": 具体价格,
    "confidence": "HIGH|MEDIUM|LOW"
}}
"#,
            kline_text,
            indicator_text,
            current_price,
            position_text,
            trend_analysis
        )
    }
    
    fn analyze_trend(&self, indicators: &TechnicalIndicators, current_price: f64) -> String {
        let rsi = indicators.rsi;
        let rsi_status = if rsi > 70.0 {
            "超买"
        } else if rsi < 30.0 {
            "超卖"
        } else {
            "中性"
        };
        
        let overall_trend = if indicators.sma_5 > indicators.sma_20 && indicators.sma_20 > indicators.sma_50 {
            "强势上涨"
        } else if indicators.sma_5 < indicators.sma_20 && indicators.sma_20 < indicators.sma_50 {
            "强势下跌"
        } else if indicators.sma_20 > indicators.sma_50 {
            "上涨趋势"
        } else if indicators.sma_20 < indicators.sma_50 {
            "下跌趋势"
        } else {
            "震荡整理"
        };
        
        let macd_direction = if indicators.macd > indicators.macd_signal {
            "多头"
        } else {
            "空头"
        };
        
        format!(
            r#"- 整体趋势: {}
- RSI状态: {:.1} ({})
- MACD方向: {}"#,
            overall_trend, rsi, rsi_status, macd_direction
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
pub struct Position {
    pub side: String,
    pub size: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
}
