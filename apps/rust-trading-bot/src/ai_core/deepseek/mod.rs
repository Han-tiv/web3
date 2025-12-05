use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ai_core::ai_trait::{
    AIProvider, EntryContext, EntryDecision, PositionContext,
    PositionDecision as AiPositionDecision, StopLossAdjustmentDecision,
    TakeProfitAdjustmentDecision,
};
use crate::prompt_contexts::{EntryPromptContext, PositionPromptContext};
use crate::valuescan_v2::TradingSignalV2;

// 引入拆分后的 prompt 模块
mod prompts;

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
            .post(format!("{}/chat/completions", self.base_url))
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
            .post(format!("{}/chat/completions", self.base_url))
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
            .post(format!("{}/chat/completions", self.base_url))
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
            .post(format!("{}/chat/completions", self.base_url))
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
            .post(format!("{}/chat/completions", self.base_url))
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

    pub fn build_entry_analysis_prompt_v2(
        &self,
        symbol: &str,
        alert_type: &str,
        alert_message: &str,
        flow_text: &str,
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
            flow_text,
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

    /// V3 Entry Prompt - 交易员思维版
    /// 整合: Valuescan关键位 + Fibonacci回撤 + 多周期共振
    /// 核心改进: 不追涨杀跌，等回撤到关键位确认反转再入场
    pub fn build_entry_analysis_prompt_v3(
        &self,
        symbol: &str,
        alert_type: &str,
        alert_message: &str,
        flow_text: &str,
        fund_type: &str,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        current_price: f64,
    ) -> String {
        let ctx = EntryPromptContext {
            symbol,
            alert_type,
            alert_message,
            flow_text,
            fund_type,
            zone_1h_summary: "",
            zone_15m_summary: "",
            entry_action: "",
            entry_reason: "",
            klines_5m,
            klines_15m,
            klines_1h,
            klines_4h: None,
            current_price,
            change_24h: None,
            signal_type: None,
            technical_indicators: None,
        };
        prompts::entry_v3::build_entry_analysis_prompt_v3(&ctx)
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
            quantity: 0.0,
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

    /// V3 Position Prompt - 交易员趋势跟踪版
    /// 核心: 顺大势、盯关键位、以趋势反转为唯一出场理由
    pub fn build_position_management_prompt_v3(
        &self,
        symbol: &str,
        side: &str,
        entry_price: f64,
        current_price: f64,
        profit_pct: f64,
        hold_duration_hours: f64,
        quantity: f64,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        indicators: &TechnicalIndicators,
        support_text: &str,
        deviation_desc: &str,
        current_stop_loss: Option<f64>,
        current_take_profit: Option<f64>,
    ) -> String {
        let ctx = PositionPromptContext {
            symbol,
            side,
            entry_price,
            current_price,
            profit_pct,
            hold_duration_hours,
            quantity,
            klines_5m,
            klines_15m,
            klines_1h,
            indicators,
            support_text,
            deviation_desc,
            current_stop_loss,
            current_take_profit,
        };
        prompts::position_v3::build_position_management_prompt_v3(&ctx)
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
