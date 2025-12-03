use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

// 引入拆分后的 prompt 模块
mod prompts;

use crate::ai::ai_trait::{
    AIProvider, EntryContext, EntryDecision, PositionContext, PositionDecision,
    StopLossAdjustmentDecision, TakeProfitAdjustmentDecision,
};
use crate::deepseek_client::{
    parse_batch_decision_response, BatchDecisionResponse, Kline, Position,
    PositionManagementDecision, TechnicalIndicators, TradingSignal,
};
use crate::prompt_contexts::{EntryPromptContext, PositionPromptContext};
use crate::valuescan_v2::{PositionManagementDecisionV2, TradingSignalV2};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: Option<i32>,
    completion_tokens: Option<i32>,
    total_tokens: Option<i32>,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        // 从环境变量读取配置，如果没有则使用默认值
        let base_url = std::env::var("GOOGLE_GEMINI_BASE_URL")
            .unwrap_or_else(|_| "https://www.packyapi.com".to_string());
        let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-pro".to_string());

        Self {
            client: Client::new(),
            api_key,
            base_url,
            model,
        }
    }

    async fn send_gemini_request(&self, prompt: &str, context_label: &str) -> Result<String> {
        self.send_gemini_request_with_model(prompt, context_label, None)
            .await
    }

    async fn send_gemini_request_with_model(
        &self,
        prompt: &str,
        context_label: &str,
        model_override: Option<&str>,
    ) -> Result<String> {
        let response_format = Some(ResponseFormat {
            format_type: "json_object".to_string(),
        });
        let request = self.build_request_with_model(prompt, model_override, response_format);

        info!("🧠 调用 Gemini API ({})...", context_label);

        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to send {} request to Gemini API", context_label))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error ({}): {}", status, error_text);
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .with_context(|| format!("Failed to parse Gemini {} response", context_label))?;

        if let Some(usage) = &openai_response.usage {
            info!(
                "✅ Gemini 响应: prompt={} | completion={} | total={}",
                usage.prompt_tokens.unwrap_or_default(),
                usage.completion_tokens.unwrap_or_default(),
                usage.total_tokens.unwrap_or_default()
            );
        } else {
            info!("✅ Gemini 响应: usage metadata unavailable");
        }

        let content = openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow!("Gemini response missing content"))?;

        info!("🔍 AI原始响应: {}", content);

        Ok(content)
    }

    fn build_request_with_model(
        &self,
        prompt: &str,
        model_override: Option<&str>,
        response_format: Option<ResponseFormat>,
    ) -> OpenAIRequest {
        OpenAIRequest {
            model: model_override
                .map(|model| model.to_string())
                .unwrap_or_else(|| self.model.clone()),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            response_format,
        }
    }

    /// 清洗 Gemini 返回的 JSON，去除 markdown 包裹，提取嵌入在文本中的 JSON
    fn clean_json_content(content: &str) -> String {
        let trimmed = content.trim();

        // 1. 处理 ```json ... ``` 格式
        if trimmed.starts_with("```json") {
            if let Some(json_content) = trimmed
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
            {
                return json_content.trim().to_string();
            }
        }

        // 2. 处理 ``` ... ``` 格式
        if trimmed.starts_with("```") {
            if let Some(json_content) = trimmed
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
            {
                return json_content.trim().to_string();
            }
        }

        // 3. 尝试从文本中提取 JSON（处理 Gemini 返回纯文本+JSON 的情况）
        // 查找第一个 { 和最后一个 }
        if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if start < end {
                let json_candidate = &trimmed[start..=end];
                // 验证是否是有效的 JSON 结构（简单检查括号配对）
                let open_braces = json_candidate.matches('{').count();
                let close_braces = json_candidate.matches('}').count();
                if open_braces == close_braces && open_braces > 0 {
                    return json_candidate.to_string();
                }
            }
        }

        // 4. 如果以上都失败，返回原始内容
        trimmed.to_string()
    }

    /// 分析市场并生成交易信号
    pub async fn analyze_market(&self, prompt: &str) -> Result<TradingSignal> {
        let content = self.send_gemini_request(prompt, "市场分析").await?;

        let cleaned_content = Self::clean_json_content(&content);
        info!("🧹 清洗后内容(市场分析): {}", cleaned_content);

        let signal: TradingSignal = match serde_json::from_str(&cleaned_content) {
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
        let content = self.send_gemini_request(prompt, "持仓管理").await?;

        let cleaned_content = Self::clean_json_content(&content);
        info!("🧹 清洗后内容(持仓管理): {}", cleaned_content);

        let decision: PositionManagementDecision = match serde_json::from_str(&cleaned_content) {
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

    /// 分析市场并生成 V2 版交易信号
    pub async fn analyze_market_v2(&self, prompt: &str) -> Result<TradingSignalV2> {
        let content = self.send_gemini_request(prompt, "市场分析V2").await?;

        let cleaned_content = Self::clean_json_content(&content);
        info!("🧹 清洗后内容(市场分析V2): {}", cleaned_content);

        let signal: TradingSignalV2 = match serde_json::from_str(&cleaned_content) {
            Ok(s) => s,
            Err(e) => {
                error!("❌ JSON解析失败: {}", e);
                error!("📄 原始内容: {}", content);
                anyhow::bail!(
                    "Failed to parse trading signal V2: {} | Raw: {}",
                    e,
                    content
                );
            }
        };

        info!(
            "📡 交易信号V2: {} | 置信度: {} | 评分: {:.1}",
            signal.signal, signal.confidence, signal.valuescan_score
        );

        Ok(signal)
    }

    /// AI 分析持仓并给出 V2 版管理决策
    pub async fn analyze_position_management_v2(
        &self,
        prompt: &str,
    ) -> Result<PositionManagementDecisionV2> {
        let content = self.send_gemini_request(prompt, "持仓管理V2").await?;

        let cleaned_content = Self::clean_json_content(&content);
        info!("🧹 清洗后内容(持仓管理V2): {}", cleaned_content);

        let decision: PositionManagementDecisionV2 = match serde_json::from_str(&cleaned_content) {
            Ok(d) => d,
            Err(e) => {
                error!("❌ JSON解析失败: {}", e);
                error!("📄 原始内容: {}", content);
                anyhow::bail!(
                    "Failed to parse position management decision V2: {} | Raw: {}",
                    e,
                    content
                );
            }
        };

        info!(
            "📊 持仓决策V2: {} | 置信度: {} | 评分: {:.1}",
            decision.action, decision.confidence, decision.valuescan_score
        );

        Ok(decision)
    }

    /// Gemini 批量持仓评估
    pub async fn evaluate_positions_batch(
        &self,
        positions: Vec<(
            String,
            String,
            f64,
            f64,
            f64,
            f64,
            Vec<Kline>,
            Vec<Kline>,
            Vec<Kline>,
            TechnicalIndicators,
        )>,
    ) -> Result<Vec<(String, PositionManagementDecision)>> {
        if positions.is_empty() {
            return Ok(vec![]);
        }

        let prompt = self.build_batch_evaluation_prompt(&positions);

        info!(
            "🧠 调用 Gemini API 进行批量持仓评估 ({} 个持仓)...",
            positions.len()
        );

        let content = self
            .send_gemini_request_with_model(&prompt, "批量持仓评估", None)
            .await?;

        let batch_response: BatchDecisionResponse = parse_batch_decision_response(&content)?;
        let BatchDecisionResponse { decisions } = batch_response;

        if decisions.len() != positions.len() {
            anyhow::bail!(
                "Batch decision count mismatch: expected {}, got {}",
                positions.len(),
                decisions.len()
            );
        }

        let mut results = Vec::with_capacity(decisions.len());

        for (idx, (position, decision)) in positions.iter().zip(decisions.iter()).enumerate() {
            let (symbol, ..) = position;
            if decision.symbol != *symbol {
                anyhow::bail!(
                    "Batch response symbol mismatch at index {}: expected {}, got {}",
                    idx,
                    symbol,
                    decision.symbol
                );
            }

            let management_decision = PositionManagementDecision {
                action: decision.action.clone(),
                close_percentage: decision.close_percentage,
                limit_price: decision.limit_price,
                reason: decision.reason.clone(),
                profit_potential: decision.profit_potential.clone(),
                optimal_exit_price: None,
                confidence: decision.confidence.clone(),
                stop_loss_adjustment: None,
                take_profit_adjustment: None,
            };

            results.push((symbol.clone(), management_decision));
        }

        info!("📦 批量持仓决策转换完成: {} 条", results.len());

        Ok(results)
    }

    /// 原样返回 Gemini 的自然语言分析内容，适合复杂自定义策略
    pub async fn analyze(&self, prompt: &str) -> Result<String> {
        self.send_gemini_request(prompt, "策略分析").await
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

    fn analyze_trend(&self, indicators: &TechnicalIndicators, _current_price: f64) -> String {
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

    pub fn build_batch_evaluation_prompt(
        &self,
        positions: &[(
            String,
            String,
            f64,
            f64,
            f64,
            f64,
            Vec<Kline>,
            Vec<Kline>,
            Vec<Kline>,
            TechnicalIndicators,
        )],
    ) -> String {
        let summarize_klines = |klines: &[Kline], limit: usize| -> Vec<Value> {
            let mut recent: Vec<&Kline> = klines.iter().rev().take(limit).collect();
            recent.reverse();
            recent
                .into_iter()
                .map(|kline| {
                    serde_json::json!({
                        "timestamp": kline.timestamp,
                        "open": kline.open,
                        "high": kline.high,
                        "low": kline.low,
                        "close": kline.close,
                        "volume": kline.volume,
                        "quote_volume": kline.quote_volume,
                        "taker_buy_volume": kline.taker_buy_volume,
                        "taker_buy_quote_volume": kline.taker_buy_quote_volume,
                    })
                })
                .collect()
        };

        let mut payload: Vec<Value> = Vec::with_capacity(positions.len());

        for (
            symbol,
            side,
            entry_price,
            current_price,
            profit_pct,
            hold_duration,
            klines_5m,
            klines_15m,
            klines_1h,
            indicators,
        ) in positions.iter()
        {
            let kline_snapshots = serde_json::json!({
                "5m": summarize_klines(klines_5m, 15),
                "15m": summarize_klines(klines_15m, 15),
                "1h": summarize_klines(klines_1h, 12),
            });

            let kline_descriptions = serde_json::json!({
                "5m": self.format_klines_with_label(klines_5m, "5m", 15),
                "15m": self.format_klines_with_label(klines_15m, "15m", 15),
                "1h": self.format_klines_with_label(klines_1h, "1h", 12),
            });

            let indicator_snapshot = serde_json::json!({
                "sma_5": indicators.sma_5,
                "sma_20": indicators.sma_20,
                "sma_50": indicators.sma_50,
                "rsi": indicators.rsi,
                "macd": indicators.macd,
                "macd_signal": indicators.macd_signal,
                "bb_upper": indicators.bb_upper,
                "bb_middle": indicators.bb_middle,
                "bb_lower": indicators.bb_lower,
            });

            payload.push(serde_json::json!({
                "symbol": symbol,
                "side": side,
                "entry_price": entry_price,
                "current_price": current_price,
                "profit_pct": profit_pct,
                "hold_duration_hours": hold_duration,
                "klines": kline_snapshots,
                "klines_text": kline_descriptions,
                "indicators": indicator_snapshot,
                "indicator_text": self.format_indicators(indicators),
                "trend_insight": self.analyze_trend(indicators, *current_price),
                "key_levels": self.identify_key_levels(klines_15m, indicators, *current_price),
            }));
        }

        let positions_json = match serde_json::to_string_pretty(&payload) {
            Ok(text) => text,
            Err(err) => {
                error!("构建批量评估 prompt JSON 失败: {}", err);
                "[]".to_string()
            }
        };

        format!(
            r#"你是资深的持仓风控分析师，请基于多时间周期K线与指标数据，为批量持仓生成纪律化决策。务必遵守超短线原则:
- 亏损 > 2% 立即止损，-0.5% ~ -1.5% 先部分减仓
- 盈利单锁定≥50%利润，再评估剩余仓位上行空间
- 禁止摊平或逆势加仓

【批量持仓数据（JSON）】
{}

【输出要求】
- 严格返回JSON数组，每个元素字段: symbol, action, close_percentage, limit_price, reason, profit_potential, confidence
- action ∈ [HOLD, PARTIAL_CLOSE, FULL_CLOSE]
- close_percentage 范围 0-100（PARTIAL/FULL 必填），limit_price 可为 null
- reason 使用精炼中文(包含趋势/关键位/指标)，profit_potential 描述剩余涨跌空间，confidence 取 HIGH|MEDIUM|LOW
- 只输出JSON，不要Markdown或额外说明

示例:
[
  {{
    "symbol": "BTCUSDT",
    "action": "PARTIAL_CLOSE",
    "close_percentage": 50,
    "limit_price": 61234.5,
    "reason": "15m 跌破 SMA20，RSI 从 70 回落",
    "profit_potential": "+3.5% 空间",
    "confidence": "MEDIUM"
  }}
]
"#,
            positions_json
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

    /// 构建开仓分析 prompt - Valuescan关键位交易法
    pub fn build_entry_analysis_prompt_v2(
        &self,
        symbol: &str,
        alert_type: &str,
        alert_message: &str,
        fund_type: &str,
        zone_1h_summary: &str,
        zone_15m_summary: &str,
        entry_action: &str,
        entry_reason: &str,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        _current_price: f64,
    ) -> String {
        let ctx = EntryPromptContext {
            symbol,
            alert_type,
            alert_message,
            fund_type,
            zone_1h_summary,
            zone_15m_summary,
            entry_action,
            entry_reason,
            klines_5m,
            klines_15m,
            klines_1h,
            klines_4h: None,
            current_price: _current_price,
            change_24h: None,
            signal_type: None,
            technical_indicators: None,
        };
        prompts::entry_v2::build_entry_analysis_prompt_v2(&ctx)
    }

    /// 构建持仓管理分析 prompt - Valuescan关键位止盈法
    pub fn build_position_management_prompt_v2(
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
        support_text: &str,
        deviation_desc: &str,
    ) -> String {
        let ctx = PositionPromptContext {
            symbol,
            side,
            entry_price,
            current_price,
            profit_pct,
            hold_duration_hours,
            klines_5m,
            klines_15m,
            klines_1h,
            indicators,
            support_text,
            deviation_desc,
            current_stop_loss: None,
            current_take_profit: None,
        };
        prompts::position_v2::build_position_management_prompt_v2(&ctx)
    }
}

#[async_trait]
impl AIProvider for GeminiClient {
    fn name(&self) -> &'static str {
        "gemini"
    }

    async fn analyze_entry(&self, ctx: &EntryContext) -> Result<EntryDecision> {
        let signal = self.analyze_market(&ctx.prompt).await?;
        Ok(EntryDecision::new(
            self.name(),
            &ctx.symbol,
            &signal.signal,
            signal.reason,
            Some(signal.confidence),
            signal.entry_price,
            signal.stop_loss,
            signal.take_profit,
            Some(ctx.metadata.clone()),
            None,
        ))
    }

    async fn analyze_position(&self, ctx: &PositionContext) -> Result<PositionDecision> {
        let PositionManagementDecision {
            action,
            close_percentage,
            limit_price,
            reason,
            profit_potential,
            optimal_exit_price,
            confidence,
            stop_loss_adjustment,
            take_profit_adjustment,
        } = self.analyze_position_management(&ctx.prompt).await?;

        Ok(PositionDecision::new(
            self.name(),
            &ctx.symbol,
            &action,
            reason,
            Some(confidence),
            Some(profit_potential),
            close_percentage,
            limit_price,
            optimal_exit_price,
            stop_loss_adjustment.map(|adj| {
                StopLossAdjustmentDecision::new(adj.should_adjust, adj.new_stop_loss, adj.reason)
            }),
            take_profit_adjustment.map(|adj| {
                TakeProfitAdjustmentDecision::new(
                    adj.should_adjust,
                    adj.new_take_profit,
                    adj.reason,
                )
            }),
            Some(ctx.metadata.clone()),
            None,
        ))
    }
}
