use anyhow::{anyhow, Result};
use log::{error, info, warn};
use rust_trading_bot::{
    deepseek_client::{DeepSeekClient, Kline, TradingSignal},
    entry_zone_analyzer::{EntryDecision, EntryZone},
    gemini_client::GeminiClient,
    signals::{AlertType, FundAlert},
    valuescan_v2::TradingSignalV2,
};
use std::sync::Arc;
use tokio::time;

use super::super::{
    modules::types::EntryPromptContext,
    trader::{build_entry_prompt_v1, build_entry_prompt_v2},
};

pub struct AIDecider {
    deepseek: Arc<DeepSeekClient>,
    gemini: Arc<GeminiClient>,
}

impl AIDecider {
    pub fn new(deepseek: Arc<DeepSeekClient>, gemini: Arc<GeminiClient>) -> Self {
        Self { deepseek, gemini }
    }

    /// 使用AI进行综合决策(K线形态优先)
    #[allow(clippy::too_many_arguments)]
    pub async fn make_trading_decision(
        &self,
        symbol: &str,
        alert: &FundAlert,
        zone_1h: &EntryZone,
        zone_15m: &EntryZone,
        entry_decision: &EntryDecision,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        current_price: f64,
        use_valuescan_v2: bool,
    ) -> Result<(
        TradingSignal,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    )> {
        info!(
            "🤖 Valuescan版本: {} (USE_VALUESCAN_V2={})",
            if use_valuescan_v2 { "V2" } else { "V1" },
            use_valuescan_v2
        );

        let alert_type_str = if alert.alert_type == AlertType::FundEscape {
            "资金出逃"
        } else {
            "资金流入"
        };

        let zone_1h_summary = format!(
            "理想价${:.4}, 范围${:.4}-${:.4}, 止损=${:.4}, 信心{:?}, 仓位{:.0}%",
            zone_1h.ideal_entry,
            zone_1h.entry_range.0,
            zone_1h.entry_range.1,
            zone_1h.stop_loss,
            zone_1h.confidence,
            zone_1h.suggested_position * 100.0
        );

        let zone_15m_summary = format!(
            "理想价${:.4}, 范围${:.4}-${:.4}, 与1h关系{:?}",
            zone_15m.ideal_entry,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1,
            zone_15m
                .relationship
                .as_ref()
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|| "未知".to_string())
        );

        let entry_action_str = format!("{:?}", entry_decision.action);

        let mut v2_score: Option<f64> = None;
        let mut v2_risk_reward: Option<f64> = None;
        let mut v2_resistance: Option<f64> = None;
        let mut v2_support: Option<f64> = None;

        let ai_signal: TradingSignal = if use_valuescan_v2 {
            let ctx = EntryPromptContext {
                symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m,
                klines_15m,
                klines_1h,
                klines_4h: None,
                current_price,
                change_24h: None,
                signal_type: None,
                technical_indicators: None,
            };

            let prompt = build_entry_prompt_v2(&ctx);

            let ai_decision_result = time::timeout(
                time::Duration::from_secs(180),
                self.deepseek.analyze_market_v2(&prompt),
            )
            .await;

            let ai_signal_v2: TradingSignalV2 = match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败(V2): {}, 跳过本次交易", e);
                    return Err(anyhow!("ai v2 decision failed"));
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s (V2), 跳过本次交易");
                    return Err(anyhow!("ai v2 decision timeout"));
                }
            };

            info!(
                "🏅 Valuescan V2评分: {:.1}/10 | 风险收益比: {:.2} | 仓位建议: {:.1}%",
                ai_signal_v2.valuescan_score,
                ai_signal_v2.risk_reward_ratio.unwrap_or(0.0),
                ai_signal_v2.position_size_pct
            );

            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                info!(
                    "   V2关键位: 阻力=${:.4} | 支撑=${:.4} | 位置={}",
                    key_levels.resistance, key_levels.support, key_levels.current_position
                );
            } else {
                info!("   V2关键位: AI未提供关键位数据");
            }

            info!(
                "🔎 AI评分详细检查: 分数={:.1}, 阈值=6.5, 动作={:?}, 理由={}",
                ai_signal_v2.valuescan_score,
                ai_signal_v2.signal,
                ai_signal_v2.reason.chars().take(50).collect::<String>()
            );

            if ai_signal_v2.valuescan_score < 6.5 {
                info!(
                    "⏸️ Valuescan V2评分{:.1}不足6.5阈值, 跳过本次交易",
                    ai_signal_v2.valuescan_score
                );
                return Err(anyhow!("valuescan score too low"));
            }

            info!("✅ Valuescan V2评分检查通过，准备执行交易逻辑");

            v2_score = Some(ai_signal_v2.valuescan_score);
            v2_risk_reward = ai_signal_v2.risk_reward_ratio;
            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                v2_resistance = Some(key_levels.resistance);
                v2_support = Some(key_levels.support);
            }

            ai_signal_v2.into()
        } else {
            let ctx = EntryPromptContext {
                symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m,
                klines_15m,
                klines_1h,
                klines_4h: None,
                current_price,
                change_24h: None,
                signal_type: None,
                technical_indicators: None,
            };

            let prompt = build_entry_prompt_v1(&ctx);

            let ai_decision_result = time::timeout(
                time::Duration::from_secs(180),
                self.gemini.analyze_market(&prompt),
            )
            .await;

            match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败: {}, 跳过本次交易", e);
                    return Err(anyhow!("ai decision failed"));
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s, 跳过本次交易");
                    return Err(anyhow!("ai decision timeout"));
                }
            }
        };

        Ok((
            ai_signal,
            v2_score,
            v2_risk_reward,
            v2_resistance,
            v2_support,
        ))
    }
}
