use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ai::ai_trait::{
    AIProvider, EntryContext, EntryDecision, PositionContext,
    PositionDecision as AiPositionDecision, StopLossAdjustmentDecision,
    TakeProfitAdjustmentDecision,
};
use crate::valuescan_v2::TradingSignalV2;

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

#[derive(Debug, Deserialize)]
pub struct BatchDecisionResponse {
    pub decisions: Vec<PositionDecision>,
}

#[derive(Debug, Deserialize)]
pub struct PositionDecision {
    pub symbol: String,
    pub action: String,
    pub close_percentage: Option<f64>,
    pub limit_price: Option<f64>,
    pub reason: String,
    pub confidence: String,
    pub profit_potential: String,
}

/// 解析批量决策响应，兼容多种 DeepSeek/Gemini JSON 输出
pub fn parse_batch_decision_response(text: &str) -> Result<BatchDecisionResponse> {
    // 清理可能的代码块标记，避免 ```json 包裹导致解析失败
    let clean_text = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match serde_json::from_str::<BatchDecisionResponse>(clean_text) {
        Ok(resp) => Ok(resp),
        Err(primary_err) => match serde_json::from_str::<Vec<PositionDecision>>(clean_text) {
            Ok(decisions) => Ok(BatchDecisionResponse { decisions }),
            Err(_) => match serde_json::from_str::<PositionDecision>(clean_text) {
                Ok(single_decision) => {
                    info!("✅ 成功解析单个持仓决策对象");
                    Ok(BatchDecisionResponse {
                        decisions: vec![single_decision],
                    })
                }
                Err(_) => {
                    error!("❌ 批量 JSON 解析失败(尝试了3种格式): {}", primary_err);
                    error!("📄 批量原始内容: {}", text);
                    anyhow::bail!(
                        "Failed to parse batch decision response: {} | Raw: {}",
                        primary_err,
                        text
                    );
                }
            },
        },
    }
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

/// Gemini API 可能缺失 profit_potential 字段，提供默认值避免解析失败
fn default_profit_potential() -> String {
    "UNKNOWN".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradingSignal {
    pub signal: String,     // "BUY", "SELL", "HOLD", "SKIP"
    pub confidence: String, // "HIGH", "MEDIUM", "LOW"
    #[serde(
        default,
        deserialize_with = "deserialize_optional_number_or_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub entry_price: Option<f64>, // AI建议的入场价 (新增)
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

/// 止损调整信息
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StopLossAdjustment {
    pub should_adjust: bool,
    pub new_stop_loss: Option<f64>,
    pub reason: String,
}

/// 止盈调整信息
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TakeProfitAdjustment {
    pub should_adjust: bool,
    pub new_take_profit: Option<f64>,
    pub reason: String,
}

/// AI持仓管理决策
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PositionManagementDecision {
    pub action: String, // "HOLD", "PARTIAL_CLOSE", "FULL_CLOSE", "SET_LIMIT_ORDER"
    pub close_percentage: Option<f64>, // 平仓百分比 (0-100)
    pub limit_price: Option<f64>, // 限价单价格
    pub reason: String,
    #[serde(default = "default_profit_potential")]
    pub profit_potential: String, // "HIGH", "MEDIUM", "LOW", "NONE"
    pub optimal_exit_price: Option<f64>, // AI判断的最优退出价
    pub confidence: String,              // "HIGH", "MEDIUM", "LOW"
    #[serde(default)]
    pub stop_loss_adjustment: Option<StopLossAdjustment>,
    #[serde(default)]
    pub take_profit_adjustment: Option<TakeProfitAdjustment>,
}

/// 增强版持仓管理分析结果
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnhancedPositionAnalysis {
    // 1. 市场趋势
    pub trend: String,                  // "UPTREND", "DOWNTREND", "SIDEWAYS"
    pub trend_confidence: f64,          // 0-100 置信度
    pub key_indicator_insights: String, // MACD/RSI/ADX 等关键信号说明

    // 2. 关键价位
    pub support_levels: Vec<f64>,    // 1-2 个支撑位
    pub resistance_levels: Vec<f64>, // 1-2 个阻力位

    // 3. 交易策略
    pub direction: String, // "LONG", "SHORT", "WAIT"
    pub entry_point: Option<f64>,
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
    pub position_adjustment: String,

    // 4. 最终动作建议
    pub recommended_actions: Vec<RecommendedAction>,

    // Legacy 字段 - 兼容现有 PositionManagementDecision
    pub action: String,
    pub reason: String,
    pub confidence: String,
    pub close_percentage: Option<f64>,
    pub limit_price: Option<f64>,
    pub profit_potential: String,
    pub optimal_exit_price: Option<f64>,
}

/// 行动参数说明
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionParams {
    pub symbol: Option<String>,
    pub side: Option<String>, // "BUY" or "SELL"
    pub quantity: Option<f64>,
    pub price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    #[serde(default)]
    pub auto_set_protection: bool, // true 表示开仓后自动设置止损/止盈保护单
    pub trigger_price: Option<f64>,
    pub order_id: Option<String>,
}

/// 推荐动作信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecommendedAction {
    pub action_type: String,
    pub priority: u8,
    pub params: ActionParams,
    pub reason: String,
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

    /// 清洗 DeepSeek 返回的 JSON，去除 markdown 包裹并提取嵌入的 JSON 片段
    fn clean_json_content(content: &str) -> String {
        let trimmed = content.trim();

        if trimmed.starts_with("```json") {
            if let Some(json_content) = trimmed
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
            {
                return json_content.trim().to_string();
            }
        }

        if trimmed.starts_with("```") {
            if let Some(json_content) = trimmed
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
            {
                return json_content.trim().to_string();
            }
        }

        if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
            if start < end {
                let json_candidate = &trimmed[start..=end];
                let open_braces = json_candidate.matches('{').count();
                let close_braces = json_candidate.matches('}').count();
                if open_braces == close_braces && open_braces > 0 {
                    return json_candidate.to_string();
                }
            }
        }

        trimmed.to_string()
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

    /// 分析市场并生成 V2 版交易信号
    pub async fn analyze_market_v2(&self, prompt: &str) -> Result<TradingSignalV2> {
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

        info!("🧠 调用 DeepSeek API (V2 信号)...");

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
            "✅ DeepSeek 响应(V2): {} tokens",
            deepseek_response.usage.total_tokens
        );

        let content = &deepseek_response.choices[0].message.content;
        info!("🔍 AI原始响应(V2): {}", content);

        let cleaned_content = Self::clean_json_content(content);
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

    /// AI 输出增强版持仓分析结构
    pub async fn analyze_position_enhanced(
        &self,
        prompt: &str,
    ) -> Result<EnhancedPositionAnalysis> {
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

        info!("🧠 调用 DeepSeek API 获取增强版持仓分析...");

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send enhanced position management request to DeepSeek API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("DeepSeek API error ({}): {}", status, error_text);
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .context("Failed to parse enhanced DeepSeek response")?;

        info!(
            "✅ DeepSeek 增强持仓响应: {} tokens",
            deepseek_response.usage.total_tokens
        );

        let content = &deepseek_response.choices[0].message.content;
        info!("🔍 增强AI原始响应: {}", content);

        let analysis: EnhancedPositionAnalysis = match serde_json::from_str(content) {
            Ok(value) => value,
            Err(e) => {
                error!("❌ 增强JSON解析失败: {}", e);
                error!("📄 原始内容: {}", content);
                anyhow::bail!(
                    "Failed to parse enhanced position analysis: {} | Raw: {}",
                    e,
                    content
                );
            }
        };

        info!(
            "🧭 趋势: {} ({:.1}%) | 策略: {} | 推荐动作: {}",
            analysis.trend,
            analysis.trend_confidence,
            analysis.direction,
            analysis.recommended_actions.len()
        );

        if let Some(action) = analysis.recommended_actions.first() {
            info!(
                "🎯 首要动作#{}, 类型: {}, 原因: {}",
                action.priority, action.action_type, action.reason
            );
        }

        Ok(analysis)
    }

    /// 批量调用 DeepSeek API 评估多个持仓，返回每个 symbol 的管理决策
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

        let request = DeepSeekRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            response_format: None,
            temperature: Some(0.7),
        };

        info!(
            "🧠 调用 DeepSeek API 进行批量持仓评估 ({} 个持仓)...",
            positions.len()
        );

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send batch position evaluation request to DeepSeek API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("DeepSeek API error ({}): {}", status, error_text);
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .context("Failed to parse DeepSeek batch response")?;

        info!(
            "✅ DeepSeek 批量响应: {} tokens",
            deepseek_response.usage.total_tokens
        );

        let content = &deepseek_response.choices[0].message.content;
        info!("🔍 批量AI原始响应: {}", content);

        let batch_response = parse_batch_decision_response(content)?;

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
            r#"你是负责 Valuescan V2 的超短线策略分析师，系统会根据你的结论直接执行自动下单，请务必严格遵守以下约束：

## 系统护栏
1. **持仓周期**：目标 30 分钟 ~ 2 小时，只做顺势延伸，不做中长线。
2. **止损纪律**：程序会把 `stop_loss` 视为硬性价格，同时在入场价 -2% 处设置兜底，禁止抗单或摊平。
3. **仓位映射**：`confidence` 决定试探仓位（HIGH=30%、MEDIUM=20%、LOW=15%），请结合信号强度合理给出。
4. **K 线优先**：5m 负责微观入场，15m 确认趋势方向，1h 仅用于定位支撑/阻力，指标信息只能作为佐证。

## 市场快照
- 当前价格: ${:.2}
- 当前持仓: {}
- 主力关键位：{}

### 5m ~ 1h K 线特征
{}

### 指标与趋势
{}

{}

## 输出任务
- 首先解读 5m/15m/1h 的组合形态、量价配合、关键影线聚集区，确认是否存在高胜率的顺势机会。
- 若已有持仓，请说明现有仓位与行情是否冲突，并优先保障风险。
- 避免笼统描述，务必写明触发你结论的具体 K 线结构/位置。

## 返回格式（JSON 对象）
{{
  "signal": "BUY" | "SELL" | "HOLD",
  "confidence": "HIGH" | "MEDIUM" | "LOW",
  "entry_price": 建议入场价(数字，基于最近有效形态),
  "stop_loss": 必须为具体数字，放在关键支撑/阻力外侧，不得留空,
  "take_profit": 必须为具体数字，基于可见阻力/回撤区，禁止简单百分比,
  "reason": "核心逻辑，需包含 5m/15m/1h 结构 + 量价/关键位，<=200字"
}}

请仅输出 JSON，不要附加解释。"#,
            current_price, position_text, key_levels, kline_text, indicator_text, trend_analysis
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

    /// 构建开仓分析 prompt - K线形态优先
    pub fn build_entry_analysis_prompt(
        &self,
        symbol: &str,
        alert_type: &str,
        alert_message: &str,
        change_24h: f64,
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
        let kline_5m_text = self.format_klines_with_label(klines_5m, "5m", 15);
        let kline_15m_text = self.format_klines_with_label(klines_15m, "15m", 15);
        let kline_1h_text = self.format_klines_with_label(klines_1h, "1h", 20);

        format!(
            r#"你是顶尖的加密货币超短线交易分析师,专注12小时内快进快出操作。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
【核心分析方法】K线形态优先,指标仅作参考
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

传统技术指标(RSI/MACD/SMA)是价格的滞后衍生物,你必须:
1. **直接分析原始K线**: 阴阳线排列、实体大小、上下影线、连续形态
2. **量价关系**: 放量突破、缩量回调、背离形态
3. **关键价格位**: 通过K线聚集识别支撑阻力,而非依赖均线
4. **多周期共振**: 5m微观入场时机 + 15m趋势确认 + 1h支撑阻力

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 【资金异动信号】(30%权重,重要参考)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- 币种: {}
- 信号类型: {} (资金流入=买入机会, 资金出逃=卖出信号)
- 24H涨跌: {:+.2}%
- 资金类型: {}
- 原始消息: {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📈 【多周期K线形态分析】(60%权重,核心决策依据)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{}

{}

{}

**K线形态分析要点**:
- **5m级别**: 最近5-10根K线的微观形态(连续阳线/阴线? 实体大小? 上下影线长度?)
  * 放量阳线突破 → 强买入信号
  * 长上影线/十字星 → 抛压沉重,谨慎
  * 连续缩量阴线 → 卖压衰竭,可能反弹
- **15m级别**: 最近10-15根K线的趋势延续性(是否形成明确方向?)
  * 连续更高的高点/低点 → 趋势确立
  * 震荡箱体突破 → 方向选择
  * 大阴线吞没前期阳线 → 趋势反转
- **1h级别**: 最近15-20根K线的支撑阻力位(K线密集区即关键位)
  * K线下影线聚集区 = 强支撑
  * K线上影线聚集区 = 强阻力
  * 当前价格与支撑阻力的相对位置?

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 【量化入场区参考】(10%权重,辅助验证)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**1h主入场区**: {}
**15m辅助入场区**: {}
**量化推荐**: {} - {}
(仅作参考,如与K线形态冲突,优先相信K线!)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【AI综合决策原则】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ **BUY信号**(开多):
- 【K线形态】5m放量阳线突破 + 15m趋势向上 (必需)
- 当前价格接近1h支撑位(K线下影线聚集区)
- 5m出现明显的反转形态(锤子线/早晨之星/多头吞没)
- 量价配合: 上涨时放量,回调时缩量
- 【资金信号】资金流入异动(加分项,非必需)

💡 **入场时机优化建议**(非强制,仅供参考):
- 建议避开RSI>70的极端超买区,等待回调至60-65再入场
- 如出现突破后立即大幅拉升(5m单根阳线>5%),建议等待回踩确认
- 价格刚创新高时可考虑等待5-10分钟观察是否出现上影线或回落
- 这些建议旨在优化入场点位,但如果K线形态和资金信号强烈,可以忽略

✅ **SELL信号**(开空):
- 【K线形态】5m放量阴线击穿 + 15m趋势向下 (必需)
- 当前价格接近1h阻力位(K线上影线聚集区)
- 5m出现顶部反转形态(流星线/黄昏之星/空头吞没)
- 量价背离: 价格新高但成交量萎缩
- 【资金信号】资金出逃信号(加分项,非必需)

💡 **做空入场时机建议**(非强制,仅供参考):
- 建议避开RSI<30的极端超卖区,等待反弹至35-40后再做空
- 如出现暴跌后单根5m阴线>5%,建议等待反弹确认压力位
- 价格刚创新低时可考虑等待是否出现下影线或反弹,避免追空
- 这些建议用于优化做空点位,但如果破位形态明显,可以忽略

❌ **SKIP条件**:
- K线形态混乱,5m/15m/1h不共振
- 当前价格在1h箱体中部,无明确支撑阻力
- 资金信号与K线形态严重冲突
- 5m出现长上下影线的十字星(犹豫形态)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "signal": "BUY|SELL|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "entry_price": 建议入场价(数字, 基于K线形态判断),
    "stop_loss": 止损价(必须为具体数字且不可为null, 基于K线形态识别并设在关键支撑/阻力下方),
    "take_profit": 止盈价(必须为具体数字且不可为null, 基于K线形态识别的关键阻力位或合理盈利目标, 禁止简单百分比估算),
    "reason": "核心决策理由(必含: K线形态描述+多周期共振+资金信号确认+止盈止损理由, 限200字)"
}}

**重要说明**:
1. confidence对应试探仓位: HIGH=30%, MEDIUM=20%, LOW=15%
2. 必须明确描述5m/15m/1h的K线形态,不能只说"趋势向上"
3. 资金信号是重要参考,但K线形态冲突时优先相信K线
4. 止损与止盈必须基于K线形态识别的支撑阻力位: 止损放在关键支撑/阻力下方,止盈设在关键阻力位或明确的合理盈利目标,严禁简单用百分比
5. stop_loss 与 take_profit 必须输出具体数字,不得返回 null、None、空字符串或占位符

现在请基于K线形态分析给出交易决策!
"#,
            symbol,
            alert_type,
            change_24h,
            fund_type,
            alert_message,
            kline_5m_text,
            kline_15m_text,
            kline_1h_text,
            zone_1h_summary,
            zone_15m_summary,
            entry_action,
            entry_reason,
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
        support_text: &str,
        deviation_desc: &str,
        current_stop_loss: Option<f64>,
        current_take_profit: Option<f64>,
        funding_rate_info: Option<(f64, f64, f64)>, // (当前费率, 平均费率, 溢价率)
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
        let stop_loss_text = current_stop_loss
            .map(|price| format!("${:.4}", price))
            .unwrap_or_else(|| "未设置".to_string());
        let take_profit_text = current_take_profit
            .map(|price| format!("${:.4}", price))
            .unwrap_or_else(|| "未设置".to_string());

        format!(
            r#"你是专业的超短线持仓管理分析师，请结合智能支撑位系统与实时偏离度执行分级止盈方案。

⚠️ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
【代码兜底规则】已自动执行,AI不需要重复判断
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

以下情况已在代码层自动处理:
- 亏损超过-5% → 自动全平 (极端止损)
⚠️ -5% 仅为系统兜底, AI 在亏损接近-3%时必须主动止损, 不要依赖极限保护。

如果持仓到达AI分析阶段,说明:
- 系统兜底条件尚未触发
- 亏损未超过-5%
- AI的任务是根据市场情况灵活判断

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

【持仓信息】
- 交易对: {}
- 持仓方向: {}
- 入场价格: ${:.4}
- 当前价格: ${:.4}
- 当前盈亏: {:+.2}%
- 持仓时长: {:.1} 小时
- 当前止损挂单价格: {}
- 当前止盈挂单价格: {}

【多周期K线快照】
{}

{}

{}

【技术指标综述】
{}

【趋势/量价洞察】
{}

【市场关键位分析】
{}
- 上方阻力位(BOLL上轨): ${:.2} (潜在上涨空间: +{:.2}%)
- 下方支撑位(BOLL下轨): ${:.2} (潜在回调风险: -{:.2}%)
- BOLL中轨: ${:.2}
- SMA50: ${:.2}

{}

【实时价格偏离度】
5m K线收盘价 vs 当前价格: {}

【AI持仓管理决策框架】基于K线形态识别平仓信号

⚠️ **优先识别的平仓信号**(按危险程度排序):

1️⃣ 【亏损止损信号 - 立即执行】⚠️⚠️⚠️
   ⚠️ 亏损持仓优先于一切其他信号:
   - 轻微亏损（-0.5% ~ -1.5%）: 5m反向吞没长阴/15m连续3根阴线/1h跌破支撑并放量/持仓>1小时且亏损扩大 → FULL_CLOSE
   - 中度亏损（-1.5% ~ -3%）: 无条件FULL_CLOSE, 不再依赖支撑位
   - 严重亏损（< -3%）: 立即FULL_CLOSE, 禁止等待-5%兜底
   - 原则: 亏损止损优先级 > 5m/15m/1h其他反转信号, 必须先止损再考虑其他策略

2️⃣ 【1h大跌信号 - 次高优先级】⚠️⚠️⚠️
   ⚠️  检查1h K线是否出现暴跌:
   - 单根1h K线跌幅>10% → 强烈建议FULL_CLOSE (见顶信号)
   - 单根1h K线跌幅>5% + 盈利>10% → 建议PARTIAL_CLOSE 70-80%
   - 从最近20根1h K线最高价回落>15% → 强烈建议FULL_CLOSE
   - 从最近20根1h K线最高价回落>10% → 建议PARTIAL_CLOSE 50-60%
   💡 1h大跌是最强反转信号,但要结合后续反弹判断

3️⃣ 【5m反转信号 - K线形态重要】
   ⚠️  检查5m K线是否出现以下形态:
   - 长上影线(上影>实体2倍) → 抛压沉重,考虑止盈
   - 倒V形态(连续3根: 低-高-低) → 价格见顶,建议止盈
   - 从最近10根5m K线的最高价回落>5% → 建议PARTIAL_CLOSE 40-50%
   - 从最近10根5m K线的最高价回落>8% → 建议FULL_CLOSE
   💡 5m回落后可能反弹,观察15m趋势是否确认

4️⃣ 【时间与盈利参考】(灵活建议,非强制)

   📌 Alpha/FOMO信号 (潜力标的):
   - 可以给更长观察期(12-24小时)
   - 盈利8-12%时考虑部分止盈30-40%
   - 盈利15%+时考虑部分止盈50-60%
   - 盈利20%+时强烈建议至少止盈70%
   - 持仓>24h且盈利<5%时考虑止盈

   📌 资金异动信号 (快进快出):
   - 建议8-12小时内结束交易
   - 盈利5-8%时考虑部分止盈30-40%
   - 盈利10%+时考虑部分止盈50-60%
   - 盈利15%+时强烈建议至少止盈70%
   - 持仓>12h且盈利<3%时考虑止盈

   💡 重要: 这些只是参考建议
   - 如果趋势强劲,可以继续持有等待更高点
   - 如果出现明确反转信号,立即止盈优先级更高
   - ZEC案例: 虽然持仓9h,但从775跌到640就应该在700+平仓

5️⃣ 【阻力位信号】
   - 距离1h阻力位<1% + 盈利>5% → 考虑PARTIAL_CLOSE 30-40%
   - 触及1h阻力位后回落 → 建议PARTIAL_CLOSE 40-50%
   - 多次触及同一阻力位未突破 → 建议止盈

✅ **继续持有条件**(需要同时满足多个):
   - 盈利<5% 且持仓<6小时
   - 5m/15m强势上涨,无明确反转K线
   - 距离1h阻力位>3%,上方空间充足
   - RSI<70 (非极端超买)
   - 没有出现1h大跌信号

⚠️ **关键判断原则**:
1. K线形态信号 > 时间/盈利建议
2. 1h大跌 > 5m回落 > 持仓时间
3. 趋势延续中可以容忍更长持仓时间
4. 出现明确反转时,立即止盈不要犹豫
5. 利润回吐>10%时,强烈建议至少部分止盈

⚠️ **亏损持仓止损规则**（优先级高于支撑位判断）:

1️⃣ **轻微亏损（-0.5% ~ -1.5%）** - 部分止损策略:
   - 5m出现反向吞没长阴线 → PARTIAL_CLOSE 40-50% (减仓观察)
   - 15m连续3根阴线 → PARTIAL_CLOSE 30-40% (趋势转弱警告)
   - 1h跌破支撑+放量 → PARTIAL_CLOSE 50% (破位信号)
   - 持仓>1小时且亏损持续扩大 → PARTIAL_CLOSE 40% (时间止损)
   - ⚠️ 部分减仓后观察5-15分钟，如继续恶化则清仓
   - 💡 给持仓一个证明机会，但已减少50%风险敞口

2️⃣ **中度亏损（-1.5% ~ -3%）**:
   - 无条件FULL_CLOSE（不再检查支撑位）
   - 5m K线持续走弱 → 立即止损，不等待
   - 理由：亏损-2%是硬止损线，AI应该在-1.5%就主动离场

3️⃣ **严重亏损（< -3%）**:
   - 立即FULL_CLOSE（无任何例外）
   - 系统兜底会在-5%强制平仓，AI必须在-3%主动执行

⚠️ **关键原则**:
- 亏损时不要幻想反弹，趋势恶化立即止损
- 5m/15m反转形态 > 1h支撑位判断
- 宁可错过反弹，也不要让小亏变大亏
- "截断亏损，让利润奔跑" - 对亏损零容忍

【输出要求】
必须严格返回一个 JSON 对象（不要 Markdown 或额外解释），字段含义如下（中文仅为提示，返回中不要包含注释文本）:
{{
    "trend": "UPTREND|DOWNTREND|SIDEWAYS，市场趋势判断",
    "trend_confidence": 0-100 的趋势置信度百分比,
    "key_indicator_insights": "说明 MACD 金叉/死叉、RSI 超买/超卖、ADX 趋势强度等关键信号",
    "support_levels": [支撑位1,支撑位2],
    "resistance_levels": [阻力位1,阻力位2],
    "direction": "LONG|SHORT|WAIT，交易策略方向",
    "entry_point": 建议入场点位(等待/观望策略时必须为 null),
    "take_profit": 建议止盈价(必须提供且不得为 null，需结合现况给出具体价位),
    "stop_loss": 建议止损价(必须提供且不得为 null，需结合现况给出具体价位),
    "position_adjustment": "仓位调整建议，说明是否需要减仓/加仓/保持",
    "recommended_actions": [
        {{
            "action_type": "IMMEDIATE_CLOSE|LIMIT_ORDER|TRIGGER_ORDER|CANCEL_TRIGGER|SET_STOP_LOSS_TAKE_PROFIT|CANCEL_STOP_LOSS_TAKE_PROFIT",
            "priority": 1-6 （1 最高，6 最低，数组需按升序排列，且遵循下述动作优先级含义）, 
            "params": {{
                "symbol": "交易对(如BTCUSDT，可为 null)",
                "side": "BUY|SELL (可为 null)",
                "quantity": 下单数量(可为 null),
                "price": 委托/执行价(可为 null),
                "stop_loss": 止损价(可为 null),
                "take_profit": 止盈价(可为 null),
                "auto_set_protection": true|false，LIMIT/TRIGGER 等开仓动作是否需要在成交后立即自动同步保护单,
                "trigger_price": 触发价(仅 TRIGGER_ORDER 需要，可为 null),
                "order_id": 取消类操作对应的原订单ID(可为 null)
            }},
            "reason": "触发该动作的中文说明，需引用趋势+关键位+指标"
        }}
    ],
    "action": "HOLD|PARTIAL_CLOSE|FULL_CLOSE|SET_LIMIT_ORDER (兼容旧版即时动作)",
    "close_percentage": 平仓百分比(当 PARTIAL_CLOSE/FULL_CLOSE 时必填 0-100，其他动作为 null),
    "limit_price": 限价/触发价(SET_LIMIT_ORDER 或触发单时必填，否则为 null),
    "reason": "综合中文理由(必须包含5m信号+15m趋势+盈亏状态+持仓时长)",
    "profit_potential": "HIGH|MEDIUM|LOW|NONE",
    "optimal_exit_price": AI判断的最优退出价(可为 null),
    "confidence": "HIGH|MEDIUM|LOW"
}}

请注意：
- 无论当前是否已有止盈/止损单，必须重新分析并给出建议的止盈止损价位，对应的 take_profit 与 stop_loss 字段禁止为 null。
- recommended_actions 中必须包含一个 SET_STOP_LOSS_TAKE_PROFIT 动作，明确说明是「新设置」还是「调整现有」，若检测到现有止盈/止损不合理（例如止损位已被突破、止盈位距离当前价过远/过近），必须提出调整方案。
- AI 必须检查当前仓位已有的止盈/止损是否合理，如需调整必须在 recommended_actions 中写明旧价位与新目标价位，确保执行侧可以据此修改。

推荐动作优先顺序（priority 数字越小越优先）：
1. IMMEDIATE_CLOSE - 趋势反转或高风险需立即平仓
2. LIMIT_ORDER - 立即挂出限价委托，可同时设置止盈/止损
3. TRIGGER_ORDER - 预测突破关键位，放置触发单（追涨杀跌）
4. CANCEL_TRIGGER - 取消不再成立的触发单
5. SET_STOP_LOSS_TAKE_PROFIT - 设置/更新现有仓位的止损止盈
6. CANCEL_STOP_LOSS_TAKE_PROFIT - 取消不匹配的止损止盈

示例:
{{
    "trend": "UPTREND",
    "trend_confidence": 82.5,
    "key_indicator_insights": "MACD 5m/15m 双金叉且 RSI 68 略高，ADX 32 表示趋势延续",
    "support_levels": [62800.0, 62250.0],
    "resistance_levels": [64150.0],
    "direction": "LONG",
    "entry_point": 63120.0,
    "take_profit": 64650.0,
    "stop_loss": 62520.0,
    "position_adjustment": "盈利回撤 2% 以内保持 60% 仓位，若跌破 62800 先减到 30%",
    "recommended_actions": [
        {{
            "action_type": "SET_STOP_LOSS_TAKE_PROFIT",
            "priority": 5,
            "params": {{
                "symbol": "BTCUSDT",
                "side": "SELL",
                "quantity": 0.8,
                "price": null,
                "stop_loss": 62520.0,
                "take_profit": 64650.0,
                "auto_set_protection": false,
                "trigger_price": null,
                "order_id": null
            }},
            "reason": "现有止损 62000/止盈 65000 偏离当前结构，建议上调至 62520/64650，锁定利润并贴合 15m 支撑"
        }},
        {{
            "action_type": "LIMIT_ORDER",
            "priority": 2,
            "params": {{
                "symbol": "BTCUSDT",
                "side": "SELL",
                "quantity": 0.3,
                "price": 64300.0,
                "stop_loss": null,
                "take_profit": null,
                "auto_set_protection": true,
                "trigger_price": null,
                "order_id": null
            }},
            "reason": "靠近 64150 阻力先行兑现部分利润"
        }}
    ],
    "action": "PARTIAL_CLOSE",
    "close_percentage": 40,
    "limit_price": null,
    "reason": "5m 出现倒V 回落 + 15m RSI 超买，当前盈利 8% 持仓 6 小时需落袋部分",
    "profit_potential": "MEDIUM",
    "optimal_exit_price": 64300.0,
    "confidence": "HIGH"
}}
"#,
            symbol,
            if side == "LONG" { "多头" } else { "空头" },
            entry_price,
            current_price,
            profit_pct,
            hold_duration_hours,
            stop_loss_text,
            take_profit_text,
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
            indicators.sma_50,
            support_text,
            deviation_desc,
        )
    }

    /// 构建批量持仓评估 prompt，要求 DeepSeek 返回 JSON 数组决策
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
            r#"你是资深的仓位风控分析师，目标是在不触碰系统底层极端止损逻辑的前提下，给出批量持仓的风险化解建议。关注超短线纪律：
- 亏损 > 2% 必须主动止损，-0.5% ~ -1.5% 先部分减仓再观察
- 盈利单需要锁定至少 50% 已实现利润，再评估剩余仓位的上行潜力
- 禁止摊平与加码逆势仓位

【批量持仓数据（JSON）】
{}

【输出要求】
- 严格返回 JSON 数组，每个元素对应一个 symbol 的 PositionDecision
- action 仅允许：HOLD、PARTIAL_CLOSE、FULL_CLOSE
- close_percentage 取值 0~100（PARTIAL_CLOSE/FULL_CLOSE 必填）
- limit_price 为建议触发价，可为 null
- reason 请用简洁中文说明（包含趋势、关键位与指标），confidence 为 HIGH|MEDIUM|LOW，profit_potential 描述剩余上涨或回撤空间
- 不要输出额外解释或 Markdown

示例:
[
  {{
    "symbol": "BTCUSDT",
    "action": "PARTIAL_CLOSE",
    "close_percentage": 50,
    "limit_price": 61234.5,
    "reason": "15m 跌破 SMA20，RSI 进入 65 高位回落",
    "confidence": "MEDIUM",
    "profit_potential": "+3.5% 空间"
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
}

// 数据结构
#[derive(Debug, Clone, Default)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,           // 成交额(USDT)
    pub taker_buy_volume: f64,       // 主动买入量
    pub taker_buy_quote_volume: f64, // 主动买入成交额(净流入)
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

#[async_trait]
impl AIProvider for DeepSeekClient {
    fn name(&self) -> &'static str {
        "deepseek"
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

    async fn analyze_position(&self, ctx: &PositionContext) -> Result<AiPositionDecision> {
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

        Ok(AiPositionDecision::new(
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
