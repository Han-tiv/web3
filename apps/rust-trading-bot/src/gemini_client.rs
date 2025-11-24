use anyhow::{anyhow, Context, Result};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::deepseek_client::{
    Kline, Position, PositionManagementDecision, TechnicalIndicators, TradingSignal,
};
use crate::valuescan_v2::{PositionManagementDecisionV2, TradingSignalV2};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
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
        let request = self.build_request(prompt);

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

    fn build_request(&self, prompt: &str) -> OpenAIRequest {
        OpenAIRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        }
    }

    /// 清洗 Gemini 返回的 JSON，去除 markdown 包裹
    fn clean_json_content(content: &str) -> String {
        let trimmed = content.trim();

        let cleaned_slice = if trimmed.starts_with("```json") {
            trimmed
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(trimmed)
        } else if trimmed.starts_with("```") {
            trimmed
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(trimmed)
        } else {
            trimmed
        };

        cleaned_slice.to_string()
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
        current_price: f64,
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

✅ **SELL信号**(开空):
- 【K线形态】5m放量阴线击穿 + 15m趋势向下 (必需)
- 当前价格接近1h阻力位(K线上影线聚集区)
- 5m出现顶部反转形态(流星线/黄昏之星/空头吞没)
- 量价背离: 价格新高但成交量萎缩
- 【资金信号】资金出逃信号(加分项,非必需)

❌ **SKIP条件**:
- K线形态混乱,5m/15m/1h不共振
- 当前价格在1h箱体中部,无明确支撑阻力
- 资金信号与K线形态严重冲突
- 5m出现长上下影线的十字星(犹豫形态)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️ 【AI入场价格约束】重要！
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

你的建议入场价 (entry_price) 必须遵守以下规则:

1. **优先考虑量化入场区**: 
   - 1h主入场区已在上文"【量化入场区参考】"段落提供
   - 15m辅助入场区已在上文"【量化入场区参考】"段落提供  
   - 量化推荐价格已在上文"【量化入场区参考】"段落提供

2. **价格偏离限制**:
   - 如果你的技术分析支持量化区,entry_price 应在量化区内
   - 如果K线形态显示强突破,entry_price 可略高于量化区上界,但:
     * 最多偏离量化区上界 +15%
     * 必须在reason中明确解释为何偏离 (如"5m放量突破关键阻力")

3. **极端情况处理**:
   - 若当前价远超量化区 >20%, 应优先给 SKIP 信号
   - 除非有明确证据表明趋势已反转 (如连续3根5m阳线+15m金叉)

4. **示例**:
   - ✅ 量化区 [$100, $110], AI建议 $108 (在区内)
   - ✅ 量化区 [$100, $110], AI建议 $115 (偏离+4.5%, 有突破依据)
   - ❌ 量化区 [$100, $110], AI建议 $130 (偏离+18%, 追高风险)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "signal": "BUY|SELL|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "entry_price": 建议入场价(数字, 基于K线形态判断),
    "stop_loss": 止损价(数字, 设在关键支撑/阻力下方),
    "reason": "核心决策理由(必含: K线形态描述+多周期共振+资金信号确认, 限200字)"
}}

**重要说明**:
1. confidence对应试探仓位: HIGH=30%, MEDIUM=20%, LOW=15%
2. 必须明确描述5m/15m/1h的K线形态,不能只说"趋势向上"
3. 资金信号是重要参考,但K线形态冲突时优先相信K线
4. 止损价必须基于K线聚集区(支撑/阻力位),不是简单的±2%

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
            r#"你是专业的超短线持仓管理分析师，请结合智能支撑位系统与实时偏离度执行分级止盈方案。

⚠️ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
【代码兜底规则】已自动执行,AI不需要重复判断
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

以下情况已在代码层自动处理:
1. 持仓4小时且未盈利(<1%) → 自动全平 (兜底保护)
2. 亏损超过-5% → 自动全平 (极端止损)

如果持仓到达AI分析阶段,说明:
- 持仓<4小时 或 已盈利>1%
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

1️⃣ 【1h大跌信号 - 最高优先级】⚠️⚠️⚠️
   ⚠️  检查1h K线是否出现暴跌:
   - 单根1h K线跌幅>10% → 强烈建议FULL_CLOSE (见顶信号)
   - 单根1h K线跌幅>5% + 盈利>10% → 建议PARTIAL_CLOSE 70-80%
   - 从最近20根1h K线最高价回落>15% → 强烈建议FULL_CLOSE
   - 从最近20根1h K线最高价回落>10% → 建议PARTIAL_CLOSE 50-60%
   💡 1h大跌是最强反转信号,但要结合后续反弹判断

2️⃣ 【5m反转信号 - K线形态重要】
   ⚠️  检查5m K线是否出现以下形态:
   - 长上影线(上影>实体2倍) → 抛压沉重,考虑止盈
   - 倒V形态(连续3根: 低-高-低) → 价格见顶,建议止盈
   - 从最近10根5m K线的最高价回落>5% → 建议PARTIAL_CLOSE 40-50%
   - 从最近10根5m K线的最高价回落>8% → 建议FULL_CLOSE
   💡 5m回落后可能反弹,观察15m趋势是否确认

3️⃣ 【时间与盈利参考】(灵活建议,非强制)

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

4️⃣ 【阻力位信号】
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

⚠️ **风险止损**:
轻微亏损（0% ~ -1.5%）: 检查Level 2支撑,距离>3%则HOLD
中度亏损（-1.5% ~ -3%）: 跌破Level 2 + 成交量增大 → FULL_CLOSE
严重亏损（< -3%）: 跌破Level 3 → FULL_CLOSE（立即离场）

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
            indicators.sma_50,
            support_text,
            deviation_desc
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
        change_24h: f64,
        fund_type: &str,
        zone_1h_summary: &str,
        zone_15m_summary: &str,
        entry_action: &str,
        entry_reason: &str,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        current_price: f64,
    ) -> String {
        let kline_5m_text = self.format_klines_with_label(klines_5m, "5m", 15);
        let kline_15m_text = self.format_klines_with_label(klines_15m, "15m", 15);
        let kline_1h_text = self.format_klines_with_label(klines_1h, "1h", 20);

        format!(
            r#"你是专业加密货币交易分析师,采用 Valuescan "关键位交易法":跟随主力资金,在关键位突破时入场。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 【资金异动信号】(30%权重,重要参考)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- 币种: {}
- 信号类型: {} (资金流入=买入机会, 资金出逃=卖出信号)
- 24H涨跌: {:+.2}%
- 资金类型: {}
- 原始消息: {}

**资金流向评分**:
- 24h资金净流入>0: +3分(强流入)
- 大单买入>55%: +2分
- 买盘主动成交>卖盘: +1分
- 主力持仓增加: +1分

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📈 【多周期K线形态分析】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{}

{}

{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【关键位判断】(50%权重,核心决策) ⭐⭐⭐⭐⭐
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**识别标准**:
1. 1h/4h K线上下影线聚集区域
2. 前期高低点(7-30天拐点)
3. 整数关口($3.00, $10.00等)
4. 成交量放大区域

**交易信号**:

✅ **突破做多**:
- 价格突破阻力 + 1h收盘确认站稳 → +3分
- 成交量≥1.5倍(主流币1.3倍) → +2分
- 回踩不破突破位 → +1分

✅ **破位做空**:
- 价格跌破支撑 + 1h收盘确认跌破 → +3分
- 成交量≥1.5倍 → +2分
- 反弹无力 → +1分

❌ **假突破**:
- 突破后<5分钟回落 OR 成交量<1.3倍 OR 反复震荡±1-2% OR 长上影线

**当前状态分析**:
```
关键阻力: ${{}} (从1h上影线聚集识别)
距离当前: {{}}%

关键支撑: ${{}} (从1h下影线聚集识别)
距离当前: {{}}%

位置判断:
距离<3%: "接近关键位,警惕"
3-10%: "安全区域,可操作"
>10%: "箱体中部,等待边界"
```

**量化入场区参考**(仅辅助验证):
- 1h主入场区: {}
- 15m辅助入场区: {}
- 量化推荐: {} - {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 【技术指标】(20%权重,辅助判断)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**多周期共振**:
- 5m: 微观入场,观察放量
- 15m: 趋势确认,观察高低点
- 1h: 主要框架,支撑阻力

**技术指标(次要)**:
- RSI: 40-60健康,>70超买,<30超卖(可长时间停留,不能单独依赖)
- MACD: 金叉辅助多,死叉辅助空(滞后性强,仅辅助)
- 成交量: 突破时≥1.5倍(主流币1.3倍,妖币2.0倍)

⚠️ **重要**: 技术指标仅辅助,关键位和资金流向是核心!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 【开仓决策规则】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**BUY 做多**(满足 3/4 条件):

**必需条件(至少 2/3)**:
1. **关键位突破**: 价格突破阻力 + 1h站稳 + 量≥1.5倍 → +3分
2. **资金流入**: 24h净流入>0 OR 大单买入>55% OR 买盘主动增加 → +2分
3. **位置合理**: 距阻力>3% AND 距支撑>2% → +2分

**加分条件(任意 1 条)**:
4. **K线配合**: 5m连续3根放量阳线 OR 15m向上 OR 1h无上影线 → +1分
5. **技术配合**: RSI 45-65 OR MACD金叉 OR 多周期一致向上 → +1分

**评分逻辑**:
- 满足2必需+1加分 且 总分≥6 → **BUY HIGH**(仓位25-30%, confidence=HIGH)
- 满足2必需 且 总分≥5 → **BUY MEDIUM**(仓位15-20%, confidence=MEDIUM)
- 否则 → **SKIP**

**风险控制**:
- 止损位: 支撑位 × 0.97(下方3%)
- 止损缓冲: 支撑位 × 0.96(下方4%)
- 风险收益比: 必须 ≥2:1

**SELL 做空**(镜像规则,不再重复)

**SKIP 观望**(出现任意 1 条):
1. **关键位模糊**: 距支撑<3% AND 距阻力<3% → 无方向,等边界
2. **震荡整理**: 反复震荡±1-2% 持续>2h → 等明确突破
3. **信号冲突**: 价突破BUT资金出(顶背离) OR 价破位BUT资金入(底背离)
4. **成交量不足**: 突破/破位 BUT 量<1.3倍 → 疑似假信号
5. **风险收益比不足**: <2:1 OR 止损>5%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "signal": "BUY|SELL|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "entry_price": 建议入场价(数字),
    "stop_loss": 止损价(数字, 支撑×0.97),
    "target_price": 目标价(数字, 可选),
    "risk_reward_ratio": 风险收益比(数字, ≥2.0),
    "position_size_pct": 仓位百分比(25.0 for HIGH, 15-20 for MEDIUM),
    "reason": "核心决策理由(必含: 关键位判断+资金流向+位置合理性+风险收益比, 限200字)",
    "key_levels": {{
        "resistance": 上方阻力价格,
        "support": 下方支撑价格,
        "current_position": "位置描述(如: 刚突破阻力,距下一阻力5.8%)"
    }},
    "valuescan_score": 总评分(0-10),
    "score_breakdown": {{
        "关键位突破": 3,
        "资金流向确认": 2,
        "位置合理": 2,
        "K线形态配合": 1,
        "技术指标配合": 0.5
    }},
    "risk_warnings": ["注意$3.30整数关口抛压", "RSI 68接近超买"],
    "coin_type": "mainstream|altcoin",
    "strategy_adjustments": {{
        "volume_threshold": 1.3,
        "stop_loss_buffer": 2.0,
        "max_hold_time": "无限制|12-24h"
    }}
}}

**字段说明**:
- **valuescan_score**: 0-10分(关键位+3,资金+2,成交量+2,形态+1,RR≥2+2)
  - ≥8.0: HIGH(仓位25-30%)
  - 6.5-7.9: MEDIUM(仓位15-20%)
  - <6.5: SKIP (代码强制,不符合开仓条件)
- **score_breakdown**: 评分透明化,必须列出各项得分
- **risk_warnings**: 必含关键风险点

**开仓检查清单(8/10才开)**:
- [ ] 1. 距关键位>3%?
- [ ] 2. 突破/破位且量>1.5倍?
- [ ] 3. 资金与价格一致?
- [ ] 4. 止损≤5%(妖币≤4%)?
- [ ] 5. RR≥2:1?
- [ ] 6. 单笔风险≤5%?
- [ ] 7. 无FOMO/恐慌?
- [ ] 8. 避开热议整数关口?
- [ ] 9. 空间>3-5%?
- [ ] 10. 最大亏损可承受?

现在请基于关键位+资金流+技术指标给出交易决策!
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
        let kline_5m_text = self.format_klines_with_label(klines_5m, "5m", 15);
        let kline_15m_text = self.format_klines_with_label(klines_15m, "15m", 15);
        let kline_1h_text = self.format_klines_with_label(klines_1h, "1h", 12);

        let indicator_text = self.format_indicators(indicators);
        let key_levels = self.identify_key_levels(klines_15m, indicators, current_price);

        let resistance = indicators.bb_upper;
        let support = indicators.bb_lower;
        let potential_upside = ((resistance - current_price) / current_price) * 100.0;
        let potential_downside = ((current_price - support) / current_price) * 100.0;

        format!(
            r#"你是专业的持仓管理分析师,基于 Valuescan 社群实战经验,核心原则: 关键位止盈,保护利润。

⚠️ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
【代码自动止损】已自动执行,AI不需要重复判断
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

以下情况已在代码层自动处理:
1. 持仓>4小时且盈利<1% → 自动全平(代码兜底)
2. 亏损>-5% → 自动全平(极端止损)
3. 跌破关键支撑Level 3 → 自动全平

如果持仓到达AI分析阶段,说明:
- 持仓<4小时 OR 已盈利>1%
- 亏损未超-5%
- AI的任务是根据市场情况灵活判断

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

【持仓信息】
- 交易对: {}
- 持仓方向: {}
- 入场价格: ${:.4}
- 当前价格: ${:.4}
- 当前盈亏: {:+.2}%
- 持仓时长: {:.1} 小时

【多周期K线快照】
{}

{}

{}

【技术指标综述】
{}

【市场关键位分析】
{}
- 上方阻力位(BOLL上轨): ${:.2} (潜在上涨空间: +{:.2}%)
- 下方支撑位(BOLL下轨): ${:.2} (潜在回调风险: -{:.2}%)

{}

【实时价格偏离度】
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【核心决策逻辑】(严格按优先级)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**优先级1(60%): 关键位止盈** ⭐⭐⭐⭐⭐

**止盈策略(距离阻力)**:
- **距阻力<1%**: PARTIAL 30-40% (接近强阻力,部分锁定,预留博突破)
- **触及阻力回落>2%**: PARTIAL 50-60% (压力有效,半仓保护)
- **突破阻力站稳**(回踩不破): HOLD (突破有效,移止损至突破位×0.99,继续持有)
- **多次触及≥3次未破**: PARTIAL 60-70% (压力太大,大概率回调)

**关键位破位止盈**:
- **跌破支撑+放量**: FULL (趋势反转,立即全平)
- **跌破支撑缩量**: PARTIAL 50% (观察假跌破,留50%)
- **回踩支撑不破**: HOLD (支撑有效,继续)

**优先级2(30%): K线反转信号** 📉

**1h级别(最高优先级)**:
- **1h跌幅>10%**: FULL (大跌见顶,全平)
- **1h跌>5% + 盈利>10%**: PARTIAL 70-80% (高位大跌,大部分止盈)
- **从1h最高回落>15%**: FULL (深度回调,趋势反转)
- **从1h最高回落>10%**: PARTIAL 50-60% (明显回调,部分保护)

**5m级别**:
- **长上影线**(上影>实体2倍): PARTIAL 30-40% (抛压重,短期回调)
- **倒V形态**(3根K:低-高-低): PARTIAL 40-50% (冲高回落,疑似顶)
- **从5m最高回落>5%**: PARTIAL 40-50% (短期回调明显)
- **从5m最高回落>8%**: FULL (大幅回落,反转信号)

**优先级3(10%): 盈利时间参考** ⏰ (灵活非强制)

**盈利梯度**:
- 5-8%: 考虑止盈20-30%(可选)
- 8-12%: 考虑止盈30-40%(建议)
- 15%+: **至少止盈50%**(强制)
- 20%+: **至少止盈70%**(强制)
- 30%+: **至少止盈90%**(强制)

**时间参考**:
- <4h且盈利>3%: 可继续(趋势强)
- >12h且盈利<3%: 考虑止盈(效率低)
- >24h且盈利<5%: 建议止盈(成本高)

⚠️ **重要**: 时间盈利仅参考,关键位和反转优先级更高!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 【持有条件】(需全部满足 5/5)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. ✓ **距阻力>3%** - 上方空间足
2. ✓ **无反转K线** - 无长上影、无倒V
3. ✓ **多周期共振** - 1h/15m/5m趋势一致向上
4. ✓ **成交量健康** - 涨时放量,回调缩量
5. ✓ **时间成本合理** - 盈利<15% OR 持仓<12h

**任意1条不满足,考虑部分止盈!**

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️ 【风险止损】(亏损时)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**轻微亏损(0~-1.5%)**:
- 距支撑>3%: HOLD (暂时安全)
- 距支撑<3%: 准备止损(警惕破位)

**中度亏损(-1.5~-3%)**:
- 跌破Level2+放量: FULL (趋势不利,立即止损)
- 未破位: HOLD (等反弹,设自动止损)

**严重亏损(-3~-5%)**:
- 跌破Level3: FULL (坚决止损,避免更大)
- 未破: 严密监控(下一支撑破位立平)

⚠️ **铁律**: 亏损>-5%由代码自动全平!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "action": "PARTIAL_CLOSE|FULL_CLOSE|HOLD",
    "close_percentage": 50.0,
    "reason": "详细分析理由(必含: 关键位判断+K线反转信号+盈亏状态+持仓时长+决策优先级, 限200字)",
    "key_analysis": {{
        "resistance_distance": "0.3%",
        "support_distance": "8.5%",
        "reversal_signals": ["1h长上影", "触及阻力"],
        "profit_level": 12.5,
        "peak_profit": 14.2,
        "drawdown": 1.7,
        "hold_duration": "6.5h"
    }},
    "optimal_exit_price": 3.30,
    "remaining_target": 3.50,
    "new_stop_loss": 3.15,
    "confidence": "HIGH",
    "valuescan_score": 8.0,
    "score_breakdown": {{
        "关键位判断": 4,
        "反转信号确认": 2,
        "盈利保护合理": 1.5,
        "风险控制到位": 0.5
    }},
    "risk_warnings": [
        "$3.30强阻力,多次触及未破",
        "盈利12%部分锁定避免回吐",
        "1h长上影显示抛压"
    ],
    "hold_conditions_check": {{
        "距离阻力>3%": false,
        "无反转K线": false,
        "多周期共振": true,
        "成交量健康": true,
        "时间成本合理": true
    }},
    "decision_priority": {{
        "level": 1,
        "reason": "关键位判断(优先级1),距阻力0.3%<1%"
    }}
}}

**字段说明**:
- **valuescan_score**: 0-10分(关键位+4,反转+3,盈利保护+2,风控+1)
  - ≥8: HIGH
  - 6-7: MEDIUM
  - <6: LOW
- **hold_conditions_check**: 5条持有条件满足情况,任意不满足→考虑止盈
- **decision_priority**: 触发哪个优先级判断(1=关键位, 2=K线反转, 3=盈利时间)

现在请基于关键位+K线反转+盈利时间给出持仓管理决策!
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
            key_levels,
            resistance,
            potential_upside,
            support,
            potential_downside,
            support_text,
            deviation_desc
        )
    }
}
