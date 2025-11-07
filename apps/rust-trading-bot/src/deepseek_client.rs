use anyhow::{Context, Result};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

fn deserialize_optional_number_or_string<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(_)) => Ok(None),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Ok(None),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradingSignal {
    pub signal: String,     // "BUY", "SELL", "HOLD"
    pub confidence: String, // "HIGH", "MEDIUM", "LOW"
    #[serde(
        default,
        deserialize_with = "deserialize_optional_number_or_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_loss: Option<f64>, // 支持数字或null
    #[serde(
        default,
        deserialize_with = "deserialize_optional_number_or_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub take_profit: Option<f64>, // 支持数字或null
    pub reason: String,
}

/// AI持仓管理决策
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PositionManagementDecision {
    pub action: String, // "HOLD", "PARTIAL_CLOSE", "FULL_CLOSE", "SET_LIMIT_ORDER"
    pub close_percentage: Option<f64>, // 平仓百分比 (0-100)
    pub limit_price: Option<f64>, // 限价单价格
    pub reason: String,
    pub profit_potential: String, // "HIGH", "MEDIUM", "LOW", "NONE"
    pub optimal_exit_price: Option<f64>, // AI判断的最优退出价
    pub confidence: String,       // "HIGH", "MEDIUM", "LOW"
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
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
            temperature: Some(0.7),
        };

        info!("🧠 调用 DeepSeek API...");

        let response = self
            .client
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

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .context("Failed to parse DeepSeek response")?;

        info!(
            "✅ DeepSeek 响应: {} tokens",
            deepseek_response.usage.total_tokens
        );

        // 解析 JSON 响应
        let content = &deepseek_response.choices[0].message.content;
        info!("🔍 AI原始响应: {}", content);

        let signal: TradingSignal = match serde_json::from_str(content) {
            Ok(s) => s,
            Err(e) => {
                error!("❌ JSON解析失败: {}", e);
                error!("📄 原始内容: {}", content);
                anyhow::bail!("Failed to parse trading signal: {} | Raw: {}", e, content);
            }
        };

        info!(
            "📡 交易信号: {} | 置信度: {}",
            signal.signal, signal.confidence
        );

        Ok(signal)
    }

    /// AI 分析持仓并给出管理决策
    pub async fn analyze_position_management(
        &self,
        prompt: &str,
    ) -> Result<PositionManagementDecision> {
        let request = DeepSeekRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
            temperature: Some(0.7),
        };

        info!("🧠 调用 DeepSeek API 进行持仓管理分析...");

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send position management request to DeepSeek API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("DeepSeek API error ({}): {}", status, error_text);
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .context("Failed to parse DeepSeek response")?;

        info!(
            "✅ DeepSeek 持仓管理响应: {} tokens",
            deepseek_response.usage.total_tokens
        );

        // 解析 JSON 响应
        let content = &deepseek_response.choices[0].message.content;
        info!("🔍 AI原始响应: {}", content);

        let decision: PositionManagementDecision = match serde_json::from_str(content) {
            Ok(d) => d,
            Err(e) => {
                error!("❌ JSON解析失败: {}", e);
                error!("📄 原始内容: {}", content);
                anyhow::bail!(
                    "Failed to parse position management decision: {} | Raw: {}",
                    e,
                    content
                );
            }
        };

        info!(
            "📊 持仓决策: {} | 盈利潜力: {} | 置信度: {}",
            decision.action, decision.profit_potential, decision.confidence
        );

        Ok(decision)
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
            r#"你是一位顶尖的加密货币交易分析师，擅长"快进快出"超短线波段交易。分析多时间周期数据：

{}

{}

【当前行情】
- 当前价格: ${:.2}
- 当前持仓: {}

🎯【超短线交易策略 - 核心原则】
1. **快进快出**: 目标持仓30分钟-2小时，不做中长线
   - 5m时间框架看入场时机，15m确认趋势方向
   - 1h仅用于识别重要支撑阻力位
2. **严格止损**: 入场后立即设置-2%硬止损，绝不抗单
   - "亏损1%立即警觉，亏损2%无条件止损"
   - 任何理由都不能成为持有亏损单的借口
3. **让利润奔跑**: 盈利时耐心持有，不设固定止盈目标
   - 等待Valuescan频道反向信号或技术反转再平仓
   - "盈利时不要急于止盈，让趋势充分发展"
4. **顺势而为**: 只做趋势延续，不抄底不摸顶
   - 5m出现反转结构信号立即平仓

{}

【当前技术状况】
{}

📊【交易决策规则 - 超短线专用】

✅ **做多入场(BUY)**:
- 5m出现快速反弹+15m上升趋势确认
- RSI 30-50区间(超卖反弹机会)
- MACD金叉且柱状线放大
- 价格站上1h支撑位且有量能配合

✅ **做空入场(SELL)**:
- 5m快速下跌+15m下降趋势确认
- RSI 50-70区间(非极端超买)
- MACD死叉且负值扩大
- 价格跌破1h阻力位转支撑为阻力

🚫 **严格止损规则**:
- 入场价-2%设置硬止损，触发立即离场
- 5m出现反向吞没K线 → 主动止损
- 持仓超1小时未盈利>1% → 主动止损
- **绝不抗单，绝不加仓摊平，绝不幻想反弹**

⚠️【超短线纪律】
- 看5m K线微观结构，出现反转立即止损
- 不因"关键位未破"而死扛亏损
- 不因"大趋势"而忽视小周期止损信号
- RSI极端值(>75或<25)优先考虑离场
- **盈利策略**: 盈利时耐心持有，等待频道反向信号或技术反转再平仓

【输出要求】
基于5m+15m+1h多周期分析，给出超短线交易决策。用JSON格式：
{{
    "signal": "BUY|SELL|HOLD",
    "reason": "简要理由(必含5m入场信号+15m趋势+1h关键位)",
    "stop_loss": 具体价格(入场价±2%),
    "take_profit": 具体价格(入场价+2.5%左右),
    "confidence": "HIGH|MEDIUM|LOW"
}}
"#,
            kline_text, indicator_text, current_price, position_text, key_levels, trend_analysis
        )
    }

    /// 识别主力关键位
    fn identify_key_levels(
        &self,
        klines: &[Kline],
        indicators: &TechnicalIndicators,
        current_price: f64,
    ) -> String {
        let bb_middle = indicators.bb_middle;
        let sma_50 = indicators.sma_50;

        // 寻找最近的高低点
        let recent_high = klines
            .iter()
            .rev()
            .take(20)
            .map(|k| k.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(current_price);

        let recent_low = klines
            .iter()
            .rev()
            .take(20)
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
            bb_middle,
            dist_to_bb_middle,
            sma_50,
            dist_to_sma50,
            recent_high,
            dist_to_high,
            recent_low,
            dist_to_low,
            key_level_status,
            if dist_to_low < 3.0 {
                "高 ⚠️"
            } else if dist_to_low < 5.0 {
                "中等"
            } else {
                "低 ✅"
            }
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

        let overall_trend = if indicators.sma_5 > indicators.sma_20
            && indicators.sma_20 > indicators.sma_50
        {
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

    /// 构建持仓管理分析 prompt - 多周期 K线
    pub fn build_position_management_prompt(
        &self,
        symbol: &str,
        side: &str,
        entry_price: f64,
        current_price: f64,
        profit_pct: f64,
        hold_duration_hours: f64,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        indicators: &TechnicalIndicators,
    ) -> String {
        // 格式化三个周期的 K线数据
        let kline_5m_text = self.format_klines_with_label(klines_5m, "5m", 15);
        let kline_15m_text = self.format_klines_with_label(klines_15m, "15m", 15);
        let kline_1h_text = self.format_klines_with_label(klines_1h, "1h", 12);

        let indicator_text = self.format_indicators(indicators);
        let trend_analysis = self.analyze_trend(indicators, current_price);
        let key_levels = self.identify_key_levels(klines_15m, indicators, current_price);

        // 计算潜在目标位
        let resistance = indicators.bb_upper;
        let support = indicators.bb_lower;
        let potential_upside = ((resistance - current_price) / current_price) * 100.0;
        let potential_downside = ((current_price - support) / current_price) * 100.0;

        format!(
            r#"你是专业的超短线持仓管理分析师，请基于"快进快出、及时止损"原则分析当前持仓。

【持仓信息】
- 交易对: {}
- 持仓方向: {}
- 入场价格: ${:.4}
- 当前价格: ${:.4}
- 当前盈亏: {:+.2}%
- 持仓时长: {:.1} 小时

{}

{}

{}

{}

【当前技术状况】
{}

{}

【市场关键位分析】
- 上方阻力位(BOLL上轨): ${:.2} (潜在上涨空间: +{:.2}%)
- 下方支撑位(BOLL下轨): ${:.2} (潜在回调风险: -{:.2}%)
- BOLL中轨: ${:.2}
- SMA50: ${:.2}

🎯【超短线持仓管理决策规则】

🚨 **立即全部平仓(FULL_CLOSE)** - 优先级最高:
- 盈亏<-1%: 已触及止损警戒线，无条件平仓
- 5m出现明显反转K线(大阴线/大阳线吞没)
- 15m趋势反转确认(MACD死叉/金叉)
- 持仓>2小时且盈亏<+1%: 超时且无盈利，离场
- **关键**: 不要幻想反弹，亏损必须果断止损

📉 **部分平仓(PARTIAL_CLOSE)** - 技术信号触发:
- 5m出现明显上影线/下影线但未完全反转
- 接近短期关键阻力/支撑位但趋势未变
- 技术指标出现背离但未确认反转
- 建议平仓百分比: 50% / 70% / 80%

✅ **继续持有(HOLD)** - 盈利时策略:
- 盈利且5m+15m趋势仍在延续
- 技术指标健康，无明显反转信号
- 距离关键阻力/支撑位仍有空间
- **重点**: 盈利时可耐心持有，等待Valuescan频道反向信号或技术反转再平仓
- **策略**: 让利润奔跑，不急于止盈，除非出现明确的反转信号

🎯 **设置限价止盈单(SET_LIMIT_ORDER)**:
- 盈利>+5%且接近强阻力位
- 可在 optimal_exit_price 设置限价单等待触发
- 用于捕捉极端波动的利润峰值

⚠️【超短线纪律 - 极其重要】
1. **亏损>-1%立即平仓**: 不找任何借口，不等待反弹
2. **持仓>2小时未盈利**: 主动离场，不浪费时间成本
3. **5m明确反转信号**: 立即响应，不看15m"大趋势"
4. **禁止抗单心态**: "关键位未破"不是持有亏损单的理由
5. **盈利持仓策略**: 盈利时可耐心持有，不设固定止盈目标，等待技术反转或频道反向信号

【分析重点】
- 首先判断5m是否出现反转信号(最高优先级)
- 其次看15m趋势是否延续
- 最后参考1h关键位
- **记住**: 超短线交易，小周期信号 > 大周期判断

【输出要求】
必须以JSON格式返回持仓管理决策:
{{
    "action": "HOLD|PARTIAL_CLOSE|FULL_CLOSE|SET_LIMIT_ORDER",
    "close_percentage": 平仓百分比(PARTIAL_CLOSE时必填,如50.0表示50%),
    "limit_price": 限价单价格(SET_LIMIT_ORDER时必填),
    "reason": "详细分析理由(必含5m信号+15m趋势+盈亏状态+持仓时长)",
    "profit_potential": "HIGH|MEDIUM|LOW|NONE",
    "optimal_exit_price": AI判断的最优退出价(可选),
    "confidence": "HIGH|MEDIUM|LOW"
}}
"#,
            symbol,
            if side == "LONG" { "多头" } else { "空头" },
            entry_price,
            current_price,
            profit_pct,
            hold_duration_hours,
            kline_5m_text,
            kline_15m_text,
            kline_1h_text,
            indicator_text,
            trend_analysis,
            key_levels,
            resistance,
            potential_upside,
            support,
            potential_downside,
            indicators.bb_middle,
            indicators.sma_50
        )
    }

    fn format_klines(&self, klines: &[Kline]) -> String {
        let mut text = String::from("【最近15根K线数据】\n");

        let recent_klines: Vec<_> = klines.iter().rev().take(15).collect();
        for (i, kline) in recent_klines.iter().rev().enumerate() {
            let trend = if kline.close > kline.open {
                "阳线"
            } else {
                "阴线"
            };
            let change = ((kline.close - kline.open) / kline.open) * 100.0;
            let body_size = ((kline.close - kline.open).abs() / kline.open) * 100.0;
            let upper_shadow = ((kline.high - kline.close.max(kline.open)) / kline.open) * 100.0;
            let lower_shadow = ((kline.open.min(kline.close) - kline.low) / kline.open) * 100.0;

            text.push_str(&format!(
                "K{:02}: {} O:{:.2} C:{:.2} H:{:.2} L:{:.2} 涨跌:{:+.2}% 实体:{:.2}% 上影:{:.2}% 下影:{:.2}%\n",
                i + 1,
                trend,
                kline.open,
                kline.close,
                kline.high,
                kline.low,
                change,
                body_size,
                upper_shadow,
                lower_shadow
            ));
        }

        text
    }

    /// 格式化K线数据，带标签（用于多周期显示）
    fn format_klines_with_label(&self, klines: &[Kline], label: &str, count: usize) -> String {
        let mut text = format!("【{}K线 - 最近{}根】\n", label, count);

        let recent_klines: Vec<_> = klines.iter().rev().take(count).collect();
        for (i, kline) in recent_klines.iter().rev().enumerate() {
            let trend = if kline.close > kline.open {
                "阳线"
            } else {
                "阴线"
            };
            let change = ((kline.close - kline.open) / kline.open) * 100.0;
            let body_size = ((kline.close - kline.open).abs() / kline.open) * 100.0;
            let upper_shadow = ((kline.high - kline.close.max(kline.open)) / kline.open) * 100.0;
            let lower_shadow = ((kline.open.min(kline.close) - kline.low) / kline.open) * 100.0;

            text.push_str(&format!(
                "K{:02}: {} O:{:.4} C:{:.4} H:{:.4} L:{:.4} 涨跌:{:+.2}% 实体:{:.2}% 上影:{:.2}% 下影:{:.2}%\n",
                i + 1,
                trend,
                kline.open,
                kline.close,
                kline.high,
                kline.low,
                change,
                body_size,
                upper_shadow,
                lower_shadow
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
                if pos.side == "long" {
                    "多头"
                } else {
                    "空头"
                },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
