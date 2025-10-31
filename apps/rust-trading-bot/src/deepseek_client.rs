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

    /// 构建分析 prompt (整合主力关键位策略)
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
        
        // 主力关键位识别
        let key_levels = self.identify_key_levels(klines, indicators, current_price);
        
        format!(
            r#"你是专业交易分析师，擅长"主力关键位策略"。分析BTC/USDT 15m数据：

{}

{}

【当前行情】
- 当前价格: ${:.2}
- 当前持仓: {}

🎯【主力关键位策略 - 核心原则】
1. **识别主力关键位**: 找出主力资金堆积的关键价格位
   - BOLL中轨/前期高低点/整数关口/成交量堆积区
2. **不破就持有**: 只要未破关键位就继续持有
   - "不破就不考虑回调" "关键位稳住就不会被甩下车"
3. **破位即止损**: 跌破主力关键位立即退出
   - "破关键位就不玩了" - 无任何犹豫
4. **二段玩法**: 突破→回踩确认→二段上涨(目标BOLL中轨)

{}

【当前技术状况】
{}

📊【交易决策规则 - 整合策略】

✅ **入场信号**:
- 价格在主力关键位附近(±2%) + 未破位 ✅
- 强势趋势(均线排列明确) + RSI合理(30-70)
- 符合"关键位附近上车"原则
- BTC做多权重可适当增加

📍 **持仓规则**:
- 主力关键位未破 → 继续持有 ✅
- 趋势延续 → 保持/加强信号
- "不破关键位就不考虑回调"

🚫 **止损规则**:
- 破关键位 → 立即退出 ❌
- 止损位 = 关键位下方2-3%
- 趋势强烈反转(需2-3指标确认)

🎯 **止盈目标**:
- 二段目标: BOLL中轨
- 前期高点/阻力位
- 预期涨幅5-10%

⚠️【防频繁交易】
- 不因单根K线改变判断
- 除非强烈反转否则保持方向
- RSI 30-70属健康范围，不过度HOLD

【输出要求】
必须明确判断主力关键位状态，给出交易信号。用JSON格式：
{{
    "signal": "BUY|SELL|HOLD",
    "reason": "简要理由(含关键位判断+趋势+技术依据)",
    "stop_loss": 具体价格,
    "take_profit": 具体价格,
    "confidence": "HIGH|MEDIUM|LOW"
}}
"#,
            kline_text,
            indicator_text,
            current_price,
            position_text,
            key_levels,
            trend_analysis
        )
    }
    
    /// 识别主力关键位
    fn identify_key_levels(&self, klines: &[Kline], indicators: &TechnicalIndicators, current_price: f64) -> String {
        let bb_middle = indicators.bb_middle;
        let sma_50 = indicators.sma_50;
        
        // 寻找最近的高低点
        let recent_high = klines.iter().rev().take(20)
            .map(|k| k.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(current_price);
        
        let recent_low = klines.iter().rev().take(20)
            .map(|k| k.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(current_price);
        
        // 计算与关键位的距离
        let dist_to_bb_middle = ((current_price - bb_middle) / bb_middle) * 100.0;
        let dist_to_sma50 = ((current_price - sma_50) / sma_50) * 100.0;
        let dist_to_high = ((recent_high - current_price) / current_price) * 100.0;
        let dist_to_low = ((current_price - recent_low) / current_price) * 100.0;
        
        // 判断关键位状态
        let key_level_status = if current_price > bb_middle && current_price > sma_50 {
            "✅ 站稳关键位上方"
        } else if current_price < bb_middle && current_price < sma_50 {
            "⚠️ 已跌破关键位"
        } else {
            "📍 在关键位附近震荡"
        };
        
        format!(
            r#"【主力关键位识别】
1. BOLL中轨: ${:.2} (距离: {:+.2}%)
2. SMA50: ${:.2} (距离: {:+.2}%)
3. 近期高点: ${:.2} (上方空间: +{:.2}%)
4. 近期低点: ${:.2} (下方距离: -{:.2}%)

关键位状态: {}
破位风险: {}"#,
            bb_middle, dist_to_bb_middle,
            sma_50, dist_to_sma50,
            recent_high, dist_to_high,
            recent_low, dist_to_low,
            key_level_status,
            if dist_to_low < 3.0 { "高 ⚠️" } else if dist_to_low < 5.0 { "中等" } else { "低 ✅" }
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
