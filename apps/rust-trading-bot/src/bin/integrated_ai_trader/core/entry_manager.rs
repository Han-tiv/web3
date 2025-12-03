use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use tokio::sync::RwLock;

use rust_trading_bot::database::{AiAnalysisRecord, Database};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::{DeepSeekClient, Kline, TradingSignal},
    entry_zone_analyzer::{EntryAction, EntryZoneAnalyzer},
    exchange_trait::ExchangeClient,
    gemini_client::GeminiClient,
    signals::{AlertType, FundAlert},
    staged_position_manager::StagedPositionManager,
    technical_analysis::TechnicalAnalyzer,
    valuescan_v2::TradingSignalV2,
};

use super::super::modules::config::USE_VALUESCAN_V2;
use super::super::modules::types::{
    EntryExecutionRequest, EntryManagerConfig, EntryPromptContext, PendingEntry, PositionTracker,
    SignalHistory, SignalRecord,
};
use super::super::utils::converters::{map_confidence_to_score, normalize_signal_type};
use super::super::utils::validators::validate_entry_zone;
use crate::trader::{build_entry_prompt_v1, build_entry_prompt_v2};

pub struct EntryManager {
    pub exchange: Arc<BinanceClient>,
    pub deepseek: Arc<DeepSeekClient>,
    pub gemini: Arc<GeminiClient>,
    pub analyzer: Arc<TechnicalAnalyzer>,
    pub entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>,
    pub signal_history: Arc<RwLock<SignalHistory>>,
    pub last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    pub max_position_usdt: f64,
    pub min_position_usdt: f64,
    pub max_leverage: u32,
    pub min_leverage: u32,
    pub db: Database,
}

impl EntryManager {
    pub fn new(cfg: EntryManagerConfig) -> Self {
        Self {
            exchange: cfg.exchange,
            deepseek: cfg.deepseek,
            gemini: cfg.gemini,
            analyzer: cfg.analyzer,
            entry_zone_analyzer: cfg.entry_zone_analyzer,
            staged_manager: cfg.staged_manager,
            position_trackers: cfg.position_trackers,
            pending_entries: cfg.pending_entries,
            signal_history: cfg.signal_history,
            last_analysis_time: cfg.last_analysis_time,
            max_position_usdt: cfg.risk_limits.max_position_usdt,
            min_position_usdt: cfg.risk_limits.min_position_usdt,
            max_leverage: cfg.risk_limits.max_leverage,
            min_leverage: cfg.risk_limits.min_leverage,
            db: cfg.db,
        }
    }

    pub async fn process_signal(&self, alert: FundAlert) -> Result<()> {
        self.analyze_and_trade(alert).await
    }

    pub async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()> {
        info!("🧠 开始AI分析: {}", alert.coin);

        // 【优化1: 信号去重】检查30秒内是否已分析过该币种
        let mut last_times = self.last_analysis_time.write().await;
        if let Some(last_time) = last_times.get(&alert.coin) {
            let elapsed = (Utc::now() - *last_time).num_seconds();
            if elapsed < 30 {
                info!("⏭️  跳过重复分析: {} ({}秒前已分析)", alert.coin, elapsed);
                return Ok(());
            }
        }
        last_times.insert(alert.coin.clone(), Utc::now());
        drop(last_times);

        // 1. 获取K线数据 - 归一化symbol为BTCUSDT格式
        let symbol = if alert.coin.ends_with("USDT") {
            alert.coin.clone()
        } else {
            format!("{}USDT", alert.coin)
        };
        info!("🔍 交易对标准化: {} -> {}", alert.coin, symbol);

        // 1.5 获取历史表现 - 12小时内交易记录
        let perf_opt = match self.exchange.get_symbol_performance(&symbol, 12).await {
            Ok(p) => p,
            Err(e) => {
                warn!("⚠️  获取{}历史表现失败: {}", symbol, e);
                None
            }
        };

        // 构建历史表现提示
        let _history_prompt = if let Some(perf) = &perf_opt {
            use rust_trading_bot::binance_client::{BinanceClient, RiskLevel};
            let risk_level = BinanceClient::get_risk_level(perf);

            info!(
                "📊 {} 历史表现(12h): {}笔交易, 胜率{:.1}%, 保证金收益率{:+.2}%, 风险等级:{:?}",
                symbol, perf.trade_count, perf.win_rate, perf.margin_loss_rate, risk_level
            );

            match risk_level {
                RiskLevel::High => format!(
                    "\n\n⚠️ 【风险警告】该币种近12小时表现糟糕：\n\
                    - 保证金亏损率: {:.2}% (严重亏损)\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    ⛔ 建议：该币种历史表现极差,强烈建议SKIP或降低置信度至LOW。\n\
                    除非有压倒性的技术优势(如明显支撑位+异动首次出现),否则不做。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Medium => format!(
                    "\n\n⚠️ 【谨慎提示】该币种近12小时表现不佳：\n\
                    - 保证金亏损率: {:.2}%\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    建议：提高决策标准,需要更强的技术信号才能开仓。信心度建议MEDIUM或以下。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Low => format!(
                    "\n\n📉 【轻度负面】该币种近12小时表现一般：\n\
                    - 保证金亏损率: {:.2}%\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    建议：略微提高警惕,按正常标准决策即可。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Normal => {
                    if perf.margin_loss_rate > 10.0 {
                        format!(
                            "\n\n✅ 【正面参考】该币种近12小时表现优秀：\n\
                            - 保证金收益率: +{:.2}%\n\
                            - 交易次数: {}笔\n\
                            - 胜率: {:.1}%\n\
                            - 总盈亏: +{:.4} USDT\n\n\
                            建议：该币种历史盈利,可以适当提高信心,但仍需结合技术面判断。",
                            perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                        )
                    } else {
                        String::new() // 轻微盈亏,不添加提示
                    }
                }
            }
        } else {
            String::new() // 无历史数据,不添加提示
        };

        // 【多时间周期分析】超短线交易策略：5m微观、15m趋势、1h支撑阻力
        let exchange = self.exchange.as_ref();
        let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                exchange.get_klines(&symbol, "5m", Some(50))
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                exchange.get_klines(&symbol, "15m", Some(100))
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                exchange.get_klines(&symbol, "1h", Some(48))
            )
        );

        // 解析5m K线
        let klines_5m = match klines_5m_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}5mK线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}5mK线超时", symbol);
                return Ok(());
            }
        };

        // 解析15m K线
        let klines = match klines_15m_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}K线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}K线超时", symbol);
                return Ok(());
            }
        };

        // 解析1h K线
        let klines_1h = match klines_1h_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}1hK线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}1hK线超时", symbol);
                return Ok(());
            }
        };

        if klines_1h.len() < 12 {
            warn!("⚠️  1h K线数据不足: {} (需要至少12根)", klines_1h.len());
            return Ok(());
        }

        if let Some(last_hour) = klines_1h.last() {
            info!(
                "🕒 1h 最新K线: 收盘价 ${:.4} | 成交量 {:.2}",
                last_hour.close, last_hour.volume
            );
        }

        if klines.len() < 20 {
            warn!("⚠️  K线数据不足: {} (需要至少20根)", klines.len());
            return Ok(());
        }

        // 2. 分析1h主入场区
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第1步: 分析1h主入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_1h = match self.entry_zone_analyzer.analyze_1h_entry_zone(&klines_1h) {
            Ok(zone) => zone,
            Err(e) => {
                warn!("❌ 1h入场区分析失败: {}", e);
                return Ok(());
            }
        };

        info!(
            "✅ 1h主入场区: 理想价格=${:.4}, 范围=${:.4}-${:.4}, 止损=${:.4}, 信心={:?}",
            zone_1h.ideal_entry,
            zone_1h.entry_range.0,
            zone_1h.entry_range.1,
            zone_1h.stop_loss,
            zone_1h.confidence
        );

        // 3. 分析15m辅助入场区
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第2步: 分析15m辅助入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_15m = match self
            .entry_zone_analyzer
            .analyze_15m_entry_zone(&klines, &zone_1h)
        {
            Ok(zone) => zone,
            Err(e) => {
                warn!("⚠️  15m辅助区分析失败: {}", e);
                return Ok(());
            }
        };

        info!(
            "✅ 15m辅助区: 理想价格=${:.4}, 范围=${:.4}-${:.4}, 关系={:?}",
            zone_15m.ideal_entry,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1,
            zone_15m.relationship
        );

        // 4. 综合决策入场策略
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🎯 第3步: 综合决策入场策略");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let current_price = klines.last().unwrap().close;
        let entry_decision =
            self.entry_zone_analyzer
                .decide_entry_strategy(&zone_1h, &zone_15m, current_price);

        info!(
            "🎯 量化决策: 动作={:?}, 价格=${:.4}, 仓位={:.0}%, 止损=${:.4}",
            entry_decision.action,
            entry_decision.price,
            entry_decision.position * 100.0,
            entry_decision.stop_loss
        );
        info!("   量化理由: {}", entry_decision.reason);

        // 4. AI综合判断 (K线形态优先)
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🤖 第4步: AI综合判断(K线形态优先)");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let alert_type_str = if alert.alert_type == AlertType::FundEscape {
            "资金出逃"
        } else {
            "资金流入"
        };

        let zone_1h_summary = format!(
            "理想价${:.4}, 范围${:.4}-${:.4}, 止损${:.4}, 信心{:?}, 仓位{:.0}%",
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
                .unwrap_or("未知".to_string())
        );

        let entry_action_str = format!("{:?}", entry_decision.action);

        let use_valuescan_v2 = *USE_VALUESCAN_V2;
        info!(
            "🤖 Valuescan版本: {} (USE_VALUESCAN_V2={})",
            if use_valuescan_v2 { "V2" } else { "V1" },
            use_valuescan_v2
        );

        // 保存V2扩展数据用于数据库记录
        let mut v2_score: Option<f64> = None;
        let mut v2_risk_reward: Option<f64> = None;
        let mut v2_resistance: Option<f64> = None;
        let mut v2_support: Option<f64> = None;

        let ai_signal: TradingSignal = if use_valuescan_v2 {
            let ctx = EntryPromptContext {
                symbol: &symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m: &klines_5m,
                klines_15m: &klines,
                klines_1h: &klines_1h,
                klines_4h: None,
                current_price,
                change_24h: None,
                signal_type: None,
                technical_indicators: None,
            };

            let prompt = build_entry_prompt_v2(&ctx);

            let ai_decision_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(180),
                self.deepseek.analyze_market_v2(&prompt),
            )
            .await;

            let ai_signal_v2: TradingSignalV2 = match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败(V2): {}, 跳过本次交易", e);
                    return Ok(());
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s (V2), 跳过本次交易");
                    return Ok(());
                }
            };

            info!(
                "🏅 Valuescan V2评分: {:.1}/10 | 风险收益比: {:.2} | 仓位建议: {:.1}%",
                ai_signal_v2.valuescan_score,
                ai_signal_v2.risk_reward_ratio.unwrap_or(0.0),
                ai_signal_v2.position_size_pct
            );

            // ✅ Bug Fix #1: 处理Optional的key_levels字段
            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                info!(
                    "   V2关键位: 阻力=${:.4} | 支撑=${:.4} | 位置={}",
                    key_levels.resistance, key_levels.support, key_levels.current_position
                );
            } else {
                info!("   V2关键位: AI未提供关键位数据");
            }

            // 【P1-3】提高Valuescan V2评分阈值,过滤低质量信号
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
                return Ok(());
            }

            info!("✅ Valuescan V2评分检查通过，准备执行交易逻辑");

            // 保存V2数据
            v2_score = Some(ai_signal_v2.valuescan_score);
            v2_risk_reward = ai_signal_v2.risk_reward_ratio;
            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                v2_resistance = Some(key_levels.resistance);
                v2_support = Some(key_levels.support);
            }

            ai_signal_v2.into()
        } else {
            let ctx = EntryPromptContext {
                symbol: &symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m: &klines_5m,
                klines_15m: &klines,
                klines_1h: &klines_1h,
                klines_4h: None,
                current_price,
                change_24h: None,
                signal_type: None,
                technical_indicators: None,
            };

            let prompt = build_entry_prompt_v1(&ctx);

            let ai_decision_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(180),
                self.gemini.analyze_market(&prompt),
            )
            .await;

            match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败: {}, 跳过本次交易", e);
                    return Ok(());
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s, 跳过本次交易");
                    return Ok(());
                }
            }
        };

        info!(
            "🎯 AI决策: {} | 信心: {} | 入场价: ${:.4} | 止损: ${:.4}",
            ai_signal.signal,
            ai_signal.confidence,
            ai_signal.entry_price.unwrap_or(current_price),
            ai_signal.stop_loss.unwrap_or(entry_decision.stop_loss)
        );
        info!("   AI理由: {}", ai_signal.reason);

        let normalized_ai_signal = ai_signal.signal.trim().to_ascii_uppercase();

        // ✅ 将AI分析写入数据库，便于前端回溯信号
        let confidence_value = map_confidence_to_score(&ai_signal.confidence);
        let entry_price_value = ai_signal.entry_price.unwrap_or(current_price);
        let stop_loss_value = ai_signal.stop_loss.unwrap_or(entry_decision.stop_loss);
        let decision_text = format!(
            "{} | 入场: ${:.4} | 止损: ${:.4}",
            ai_signal.signal, entry_price_value, stop_loss_value
        );
        let signal_type = normalize_signal_type(&ai_signal.signal);
        let ai_record = AiAnalysisRecord {
            id: None,
            timestamp: Utc::now().to_rfc3339(),
            symbol: symbol.clone(),
            decision: decision_text,
            confidence: confidence_value,
            signal_type: Some(signal_type.to_string()),
            reason: ai_signal.reason.clone(),
            valuescan_score: v2_score,
            risk_reward_ratio: v2_risk_reward,
            entry_price: Some(entry_price_value),
            stop_loss: Some(stop_loss_value),
            resistance: v2_resistance,
            support: v2_support,
        };

        if let Err(e) = self.db.insert_ai_analysis(&ai_record) {
            warn!("⚠️  保存AI分析到数据库失败: {}", e);
        }

        // 根据AI决策过滤 - 只过滤SKIP信号,不再强制过滤资金信号矛盾
        match normalized_ai_signal.as_str() {
            "SKIP" => {
                info!("\n⏸️  AI建议跳过: {}", ai_signal.reason);

                // 加入延迟开仓队列，等待后续重新评估
                let mut pending = self.pending_entries.write().await;
                if let Some(existing) = pending.get_mut(&symbol) {
                    existing.retry_count += 1;
                    existing.last_analysis_time = Utc::now();
                    existing.reject_reason = format!("AI SKIP: {}", ai_signal.reason);
                    let retry_count = existing.retry_count;
                    drop(pending);
                    info!(
                        "📝 {} 已在延迟队列中，更新重试次数: {}",
                        symbol, retry_count
                    );
                } else {
                    pending.insert(
                        symbol.clone(),
                        PendingEntry {
                            symbol: symbol.clone(),
                            first_signal_time: Utc::now(),
                            last_analysis_time: Utc::now(),
                            alert: alert.clone(),
                            reject_reason: format!("AI SKIP: {}", ai_signal.reason),
                            retry_count: 0,
                            fund_escape_detected_at: None,
                        },
                    );
                    drop(pending);
                    info!("📝 已加入延迟开仓队列: {} (AI SKIP)", symbol);
                }

                return Ok(());
            }
            "BUY" | "SELL" => {
                // ✅ AI已综合资金信号+K线形态做出判断,直接执行
                info!(
                    "✅ AI综合判断: {} (资金信号: {})",
                    ai_signal.signal, alert_type_str
                );
            }
            _ => {
                warn!("⚠️  未知AI信号: {}, 跳过", ai_signal.signal);
                return Ok(());
            }
        }

        // 5. 执行试探建仓 (使用AI微调后的价格)
        let final_entry_price = ai_signal.entry_price.unwrap_or(entry_decision.price);
        let side = if normalized_ai_signal.eq_ignore_ascii_case("SELL") {
            "SHORT"
        } else {
            "LONG"
        };

        // 使用 EntryDecision 风险区间 + 杠杆 推导方向感知止损
        let leverage_for_stop = match zone_1h.confidence {
            rust_trading_bot::entry_zone_analyzer::Confidence::High => self.max_leverage,
            rust_trading_bot::entry_zone_analyzer::Confidence::Medium => {
                (self.min_leverage + self.max_leverage) / 2
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Low => self.min_leverage,
        } as u32;
        let risk_pct = if entry_decision.price <= 0.0 {
            0.005
        } else {
            ((entry_decision.price - entry_decision.stop_loss) / entry_decision.price)
                .abs()
                .max(0.005)
        };
        let direction_aware_stop_loss = if side == "LONG" {
            final_entry_price * (1.0 - risk_pct)
        } else {
            final_entry_price * (1.0 + risk_pct)
        };
        let leverage_f64 = leverage_for_stop.max(1) as f64;
        let liquidation_price = if side == "LONG" {
            final_entry_price * (1.0 - 1.0 / leverage_f64)
        } else {
            final_entry_price * (1.0 + 1.0 / leverage_f64)
        };
        let safe_stop_loss = if side == "LONG" {
            direction_aware_stop_loss.max(liquidation_price * 1.01)
        } else {
            direction_aware_stop_loss.min(liquidation_price * 0.99)
        };
        let final_stop_loss = ai_signal.stop_loss.unwrap_or(safe_stop_loss);
        info!(
            "💡 {} 止损计算: 入场=${:.8}, 风险={:.2}%, 方向止损=${:.8}, 爆仓价=${:.8}, 最终止损=${:.8}",
            symbol,
            final_entry_price,
            risk_pct * 100.0,
            direction_aware_stop_loss,
            liquidation_price,
            final_stop_loss
        );

        let final_confidence = &ai_signal.confidence;

        // 根据AI confidence调整仓位比例
        let ai_position_multiplier = match final_confidence.as_str() {
            "HIGH" => 1.0,    // 30%全额
            "MEDIUM" => 0.67, // 20%
            "LOW" => 0.5,     // 15%
            _ => 1.0,
        };

        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("💰 第5步: 执行试探建仓");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let build_exec_request = |is_ai_override: bool| EntryExecutionRequest {
            symbol: &symbol,
            alert: &alert,
            zone_1h: &zone_1h,
            entry_decision: &entry_decision,
            klines_15m: &klines,
            klines_5m: &klines_5m,
            current_price,
            final_entry_price,
            final_stop_loss,
            final_confidence: final_confidence.as_str(),
            ai_position_multiplier,
            ai_signal_side: normalized_ai_signal.as_str(),
            take_profit: ai_signal.take_profit,
            is_ai_override,
        };

        // 根据决策动作执行
        match entry_decision.action {
            EntryAction::EnterNow | EntryAction::EnterWithCaution => {
                self.execute_ai_trial_entry(build_exec_request(false))
                    .await?;
            }
            EntryAction::WaitForPullback => {
                let ai_trade_signal = matches!(normalized_ai_signal.as_str(), "BUY" | "SELL");
                let ai_high_confidence = ai_signal.confidence.trim().eq_ignore_ascii_case("HIGH");

                if ai_trade_signal && ai_high_confidence {
                    warn!("⚠️  量化建议等待回调,但AI HIGH信心覆盖决策");
                    info!("   量化理由: {}", entry_decision.reason);
                    info!(
                        "   AI信心: {} | 信号: {} | 理由: {}",
                        ai_signal.confidence, ai_signal.signal, ai_signal.reason
                    );

                    self.execute_ai_trial_entry(build_exec_request(true))
                        .await?;
                } else {
                    info!("\n📌 等待回调到更好价格: ${:.4}", entry_decision.price);
                    info!("   理由: {}", entry_decision.reason);
                    info!("   AI信心不足以覆盖量化决策,暂不执行");

                    // 加入延迟开仓队列 - 等待回调确认
                    let mut pending = self.pending_entries.write().await;
                    if let Some(existing) = pending.get_mut(&symbol) {
                        existing.retry_count += 1;
                        existing.last_analysis_time = Utc::now();
                        existing.reject_reason = format!("等待回调: {}", entry_decision.reason);
                        let retry_count = existing.retry_count;
                        drop(pending);
                        info!(
                            "📝 {} 已在延迟队列中，更新重试次数: {}",
                            symbol, retry_count
                        );
                    } else {
                        pending.insert(
                            symbol.clone(),
                            PendingEntry {
                                symbol: symbol.clone(),
                                first_signal_time: Utc::now(),
                                last_analysis_time: Utc::now(),
                                alert: alert.clone(),
                                reject_reason: format!("等待回调: {}", entry_decision.reason),
                                retry_count: 0,
                                fund_escape_detected_at: None,
                            },
                        );
                        drop(pending);
                        info!("📝 已加入延迟开仓队列: {} (等待回调)", symbol);
                    }
                }
            }
            EntryAction::Skip => {
                info!("\n⏸️  入场条件不佳,跳过本次信号");
                info!("   理由: {}", entry_decision.reason);
            }
        }

        Ok(())
    }

    pub async fn execute_ai_trial_entry(&self, req: EntryExecutionRequest<'_>) -> Result<()> {
        let EntryExecutionRequest {
            symbol,
            alert,
            zone_1h,
            entry_decision,
            klines_15m,
            klines_5m,
            current_price,
            final_entry_price,
            final_stop_loss,
            final_confidence,
            ai_position_multiplier,
            ai_signal_side,
            take_profit,
            is_ai_override,
        } = req;
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("💰 第4步: 执行试探建仓");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // ✅ 使用AI判断的方向(BUY/SELL),不再强制根据资金信号决定
        let side = if ai_signal_side.eq_ignore_ascii_case("SELL") {
            "SHORT"
        } else {
            "LONG"
        };
        let mut stop_loss_order_id: Option<String> = None;
        let mut take_profit_order_id: Option<String> = None;

        // 动态计算杠杆和仓位
        let (position_usdt, leverage) = match zone_1h.confidence {
            rust_trading_bot::entry_zone_analyzer::Confidence::High => {
                (self.max_position_usdt, self.max_leverage)
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Medium => {
                let mid_position = (self.min_position_usdt + self.max_position_usdt) / 2.0;
                let mid_leverage = (self.min_leverage + self.max_leverage) / 2;
                (mid_position, mid_leverage)
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Low => {
                (self.min_position_usdt, self.min_leverage)
            }
        };

        // 计算试探仓位数量 (使用AI微调后的价格和仓位)
        let adjusted_position = entry_decision.position * ai_position_multiplier;
        // 先基于交易规则动态校验名义金额，避免低于交易所门槛
        let rules = self.exchange.get_symbol_trading_rules(symbol).await?;
        let min_notional = rules.min_notional.unwrap_or(5.0);
        let base_notional = position_usdt * leverage as f64 * adjusted_position;
        let required_notional = if base_notional < min_notional {
            let adjusted = min_notional * 1.05;
            warn!(
                "📊 {} 基础名义金额 {:.2} USDT < 最低要求 {:.2} USDT, 自动提升到 {:.2} USDT",
                symbol, base_notional, min_notional, adjusted
            );
            adjusted
        } else {
            base_notional
        };
        let trial_quantity = required_notional / final_entry_price;

        info!("💰 试探建仓配置:");
        info!(
            "   AI信心度: {} → 仓位调整: {:.0}%",
            final_confidence,
            adjusted_position * 100.0
        );
        info!("   投入USDT: {:.2}", position_usdt);
        info!("   杠杆倍数: {}x", leverage);
        info!("   开仓数量: {:.6} {}", trial_quantity, alert.coin);
        info!("   入场价格: ${:.4} (AI微调)", final_entry_price);
        info!("   止损价格: ${:.4} (AI微调)", final_stop_loss);

        // 【P0-2】入场区验证 - 拒绝追高
        // 使用最新5m K线收盘价作为信号价，避免 alert.price 恒为 0 造成 inf 偏离
        let signal_price = klines_5m.last().map(|k| k.close).unwrap_or(current_price);
        let entry_zone = (zone_1h.entry_range.0, zone_1h.entry_range.1);
        let indicators = self.analyzer.calculate_indicators(klines_15m);

        if !validate_entry_zone(
            signal_price,
            final_entry_price,
            entry_zone,
            &indicators,
            is_ai_override,
        )
        .await?
        {
            warn!("⚠️ 入场区验证失败，跳过建仓");

            // 加入延迟开仓队列 - 当前价格不在入场区
            let symbol_owned = symbol.to_string();
            let mut pending = self.pending_entries.write().await;
            if let Some(existing) = pending.get_mut(symbol) {
                existing.retry_count += 1;
                existing.last_analysis_time = Utc::now();
                existing.reject_reason = "价格不在入场区".to_string();
                let retry_count = existing.retry_count;
                drop(pending);
                info!(
                    "📝 {} 已在延迟队列中，更新重试次数: {}",
                    symbol, retry_count
                );
            } else {
                pending.insert(
                    symbol_owned.clone(),
                    PendingEntry {
                        symbol: symbol_owned,
                        first_signal_time: Utc::now(),
                        last_analysis_time: Utc::now(),
                        alert: alert.clone(),
                        reject_reason: "价格不在入场区".to_string(),
                        retry_count: 0,
                        fund_escape_detected_at: None,
                    },
                );
                drop(pending);
                info!("📝 已加入延迟开仓队列: {} (价格不符)", symbol);
            }

            return Ok(());
        }

        info!("✅ 入场区验证通过，继续执行建仓");

        // 设置杠杆和交易模式
        info!(
            "⚙️  设置交易模式: 杠杆={}x, 保证金=逐仓, 模式=单向",
            leverage
        );
        if let Err(e) = self
            .exchange
            .ensure_trading_modes(symbol, leverage, "ISOLATED", false)
            .await
        {
            error!("❌ 设置交易模式失败: {}", e);
            return Err(e);
        }

        // 限价单入场
        let order_side = if side == "LONG" { "BUY" } else { "SELL" };
        match self
            .exchange
            .limit_order(
                symbol,
                trial_quantity,
                order_side,
                final_entry_price,
                Some(side),
                false,
            )
            .await
        {
            Ok(order_id) => {
                info!("✅ 试探建仓订单已提交: {}", order_id);

                let canceled_orders = match self.cancel_symbol_trigger_orders(symbol).await {
                    Ok(ids) => ids,
                    Err(e) => {
                        warn!("⚠️  清理旧触发单失败: {}", e);
                        Vec::new()
                    }
                };
                info!(
                    "🗑️ 取消旧触发单 {} 个: {:?}",
                    canceled_orders.len(),
                    canceled_orders
                );

                // 设置止损挂单
                match self
                    .exchange
                    .set_stop_loss(symbol, side, trial_quantity, final_stop_loss, None)
                    .await
                {
                    Ok(sl_order_id) => {
                        info!(
                            "✅ 止损单已设置 @ ${:.4}, 订单ID: {}",
                            final_stop_loss, sl_order_id
                        );
                        stop_loss_order_id = Some(sl_order_id);
                    }
                    Err(e) => {
                        warn!("⚠️  止损单设置失败: {}", e);
                    }
                }

                // 设置止盈挂单(如果AI提供了take_profit)
                if let Some(take_profit_price) = take_profit {
                    match self
                        .exchange
                        .set_limit_take_profit(symbol, side, trial_quantity, take_profit_price)
                        .await
                    {
                        Ok(tp_order_id) => {
                            info!(
                                "✅ 止盈单已设置 @ ${:.4}, 订单ID: {}",
                                take_profit_price, tp_order_id
                            );
                            take_profit_order_id = Some(tp_order_id);
                        }
                        Err(e) => {
                            warn!("⚠️  止盈单设置失败: {}", e);
                        }
                    }
                } else {
                    info!("ℹ️  AI未提供止盈价,不设置止盈挂单");
                }

                // 成功开仓，从延迟队列移除
                {
                    let mut pending = self.pending_entries.write().await;
                    if pending.remove(symbol).is_some() {
                        info!("✅ {} 成功开仓，已从延迟队列移除", symbol);
                    }
                }

                // 创建试探持仓记录 (使用AI微调后的entry_decision)
                let mut adjusted_entry_decision = entry_decision.clone();
                adjusted_entry_decision.price = final_entry_price;
                adjusted_entry_decision.stop_loss = final_stop_loss;
                adjusted_entry_decision.position = adjusted_position;

                let mut staged_manager = self.staged_manager.write().await;
                match staged_manager.create_trial_position(
                    symbol.to_string(),
                    side.to_string(),
                    &adjusted_entry_decision,
                    position_usdt,
                    leverage as f64,
                ) {
                    Ok(_) => {
                        info!("✅ 试探持仓已记录,等待启动信号补仓70%");

                        // ✅ 新增: 同时记录到 position_trackers，让AI能监控平仓
                        let mut trackers = self.position_trackers.write().await;
                        trackers.insert(
                            symbol.to_string(),
                            PositionTracker {
                                symbol: symbol.to_string(),
                                entry_price: final_entry_price,
                                quantity: trial_quantity,
                                leverage,
                                side: side.to_string(),
                                stop_loss_order_id: stop_loss_order_id.clone(),
                                take_profit_order_id: take_profit_order_id.clone(),
                                entry_time: Utc::now(),
                                last_check_time: Utc::now(),
                            },
                        );
                        info!("✅ 持仓已同步到AI监控系统 (双轨记录)");
                        drop(trackers);

                        // 记录信号历史
                        let signal_record = SignalRecord {
                            timestamp: Utc::now().to_rfc3339(),
                            signal: if side == "LONG" {
                                "BUY".to_string()
                            } else {
                                "SELL".to_string()
                            },
                            confidence: "MEDIUM".to_string(),
                            reason: format!("试探建仓: {}", entry_decision.reason.clone()),
                            price: entry_decision.price,
                        };
                        self.signal_history.write().await.add(signal_record);
                    }
                    Err(e) => {
                        error!("❌ 创建试探持仓记录失败: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ 试探建仓订单提交失败: {}", e);
            }
        }

        Ok(())
    }

    async fn cancel_symbol_trigger_orders(&self, symbol: &str) -> Result<Vec<u64>> {
        // 先快照当前 tracker，避免持锁执行异步请求
        let tracker_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.get(symbol).cloned()
        };

        let Some(tracker) = tracker_snapshot else {
            return Ok(Vec::new());
        };

        let mut targets: Vec<(&str, String)> = Vec::new();
        if let Some(order_id) = tracker.stop_loss_order_id.clone() {
            targets.push(("止损", order_id));
        }
        if let Some(order_id) = tracker.take_profit_order_id.clone() {
            targets.push(("止盈", order_id));
        }

        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let mut canceled_raw: Vec<String> = Vec::new();

        for (order_type, order_id) in targets {
            match self.exchange.cancel_order(symbol, &order_id).await {
                Ok(_) => {
                    info!("🧹 {} 旧{}单已取消: {}", symbol, order_type, order_id);
                    canceled_raw.push(order_id);
                }
                Err(err) => {
                    warn!(
                        "⚠️  {} 旧{}单取消失败 (order_id={}): {}",
                        symbol, order_type, order_id, err
                    );
                }
            }
        }

        if canceled_raw.is_empty() {
            return Ok(Vec::new());
        }

        {
            let mut trackers = self.position_trackers.write().await;
            if let Some(tracker) = trackers.get_mut(symbol) {
                if tracker
                    .stop_loss_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.stop_loss_order_id = None;
                }
                if tracker
                    .take_profit_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.take_profit_order_id = None;
                }
                tracker.last_check_time = Utc::now();
            }
        }

        let mut canceled_numeric = Vec::new();
        for raw in canceled_raw {
            match raw.parse::<u64>() {
                Ok(id) => canceled_numeric.push(id),
                Err(_) => {
                    warn!(
                        "⚠️  order_id 无法转换为数字 (symbol={}, raw={})，仍视为已清理",
                        symbol, raw
                    );
                }
            }
        }

        Ok(canceled_numeric)
    }
}
