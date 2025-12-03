use anyhow::Result;
use log::{info, warn};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::Kline,
    exchange_trait::ExchangeClient,
    support_analyzer::{Kline as SupportKline, SupportAnalysisRequest, SupportAnalyzer},
    technical_analysis::TechnicalAnalyzer,
};
use std::sync::Arc;
use tokio::time;

use super::super::utils::validators::is_meme_coin;
use super::super::{
    PositionAction, PositionContextRequest, PositionEvaluationStep, PositionMarketContext,
    PreparedPositionContext,
};

/// 负责构建 AI 持仓评估上下文，并在早期直接给出强制动作
pub struct ContextBuilder {
    exchange: Arc<BinanceClient>,
    analyzer: Arc<TechnicalAnalyzer>,
}

impl ContextBuilder {
    pub fn new(exchange: Arc<BinanceClient>, analyzer: Arc<TechnicalAnalyzer>) -> Self {
        Self { exchange, analyzer }
    }

    /// 暴露给外部方便复用 exchange 引用
    pub fn exchange(&self) -> &Arc<BinanceClient> {
        &self.exchange
    }

    pub async fn prepare_position_context(
        &self,
        req: PositionContextRequest<'_>,
    ) -> Result<PositionEvaluationStep> {
        // 从request解构参数
        let symbol = req.symbol;
        let side = req.side;
        let entry_price = req.entry_price;
        let stop_loss_price = req.stop_loss_price;
        let current_price = req.current_price;
        let quantity = req.quantity;
        let duration = req.duration_hours;
        let stop_loss_order_id = req.stop_loss_order_id;
        let take_profit_order_id = req.take_profit_order_id;

        let profit_pct = if side == "LONG" {
            ((current_price - entry_price) / entry_price) * 100.0
        } else {
            ((entry_price - current_price) / entry_price) * 100.0
        };

        if profit_pct >= 15.0 {
            info!(
                "💰 {} 盈利已达 {:+.2}% >= 15%, 触发强制全仓平仓 (锁定利润)",
                symbol, profit_pct
            );
            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: "profit_target_15pct".to_string(),
                },
            ));
        }

        if profit_pct >= 10.0 && duration >= 2.0 {
            info!(
                "⏰ {} 盈利 {:+.2}% >= 10% 且持仓 {:.1}h >= 2h, 触发强制全仓平仓 (时间效率)",
                symbol, profit_pct, duration
            );
            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: "profit_time_optimization".to_string(),
                },
            ));
        }

        info!(
            "🤖 {} 当前盈亏 {:+.2}%, 调用 AI 评估持仓管理...",
            symbol, profit_pct
        );

        let market_context = match self.collect_position_market_context(symbol).await? {
            Some(ctx) => ctx,
            None => return Ok(PositionEvaluationStep::Skip),
        };

        let mut min_notional_cache: Option<f64> = None;

        if market_context.klines_1h.len() >= 3 {
            let last_3_candles = &market_context.klines_1h[market_context.klines_1h.len() - 3..];
            let all_opposite = if side == "LONG" {
                last_3_candles.iter().all(|k| k.close < k.open)
            } else {
                last_3_candles.iter().all(|k| k.close > k.open)
            };

            if all_opposite {
                let opposite_type = if side == "LONG" { "阴线" } else { "阳线" };
                let close_pct = if profit_pct >= 10.0 {
                    70.0
                } else if profit_pct >= 5.0 {
                    60.0
                } else {
                    50.0
                };

                warn!(
                    "📉 {} 触发P0-1规则: 连续3根1h{} (Valuescan止盈信号)",
                    symbol, opposite_type
                );
                warn!(
                    "   持仓方向: {} | 当前盈亏: {:+.2}% | 建议止盈: {:.0}%",
                    side, profit_pct, close_pct
                );

                let close_quantity = (quantity * (close_pct / 100.0)).clamp(0.0, quantity);
                let min_notional = self
                    .resolve_min_notional(symbol, &mut min_notional_cache)
                    .await?;

                let market_price = match self.exchange.get_current_price(symbol).await {
                    Ok(price) => price,
                    Err(_) => entry_price,
                };

                let position_total_value = quantity * market_price;
                let suggested_close_value = close_quantity * market_price;

                if suggested_close_value < min_notional {
                    let min_ratio_pct = (min_notional / position_total_value * 100.0).ceil();

                    if min_ratio_pct <= 100.0 {
                        let adjusted_close_pct = min_ratio_pct;
                        let adjusted_close_qty = quantity * (adjusted_close_pct / 100.0);
                        let adjusted_remaining = (quantity - adjusted_close_qty).max(0.0);

                        warn!(
                            "⚠️ {} 部分平仓比率调整: AI建议{:.0}% (${:.2}) → 实际执行{:.0}% (${:.2})，满足MIN_NOTIONAL ${:.0}",
                            symbol, close_pct, suggested_close_value, adjusted_close_pct, adjusted_close_qty * market_price, min_notional
                        );

                        return Ok(PositionEvaluationStep::Immediate(
                            PositionAction::PartialClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                close_quantity: adjusted_close_qty,
                                close_pct: adjusted_close_pct,
                                entry_price,
                                stop_loss_price,
                                remaining_quantity: adjusted_remaining,
                                stop_loss_order_id: stop_loss_order_id.clone(),
                            },
                        ));
                    } else {
                        warn!(
                            "⚠️ {} 持仓总价值(${:.2}) < MIN_NOTIONAL(${:.0})，无法部分平仓，执行全部平仓",
                            symbol, position_total_value, min_notional
                        );
                        return Ok(PositionEvaluationStep::Immediate(
                            PositionAction::FullClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                quantity,
                                reason: "valuescan_p0_1_min_notional_full_close".to_string(),
                            },
                        ));
                    }
                }

                let remaining_quantity = (quantity - close_quantity).max(0.0);
                return Ok(PositionEvaluationStep::Immediate(
                    PositionAction::PartialClose {
                        symbol: symbol.to_string(),
                        side: side.to_string(),
                        close_quantity,
                        close_pct,
                        entry_price,
                        stop_loss_price,
                        remaining_quantity,
                        stop_loss_order_id: stop_loss_order_id.clone(),
                    },
                ));
            }
        }

        let is_meme = is_meme_coin(symbol);
        let time_limit_hours = if is_meme { 4.0 } else { 8.0 };

        if duration >= time_limit_hours {
            warn!(
                "⏰ {} 触发P0-2规则: 持仓{:.1}h >= {:.0}h ({}流动性时间窗口)",
                symbol,
                duration,
                time_limit_hours,
                if is_meme { "MEME币" } else { "山寨币" }
            );
            warn!("   Valuescan核心理论: 流动性最多维持4-8h, 超时强制退出");

            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: format!("time_limit_{}h", time_limit_hours as u32),
                },
            ));
        }

        if market_context.klines_1h.len() >= 2 {
            let current_candle = &market_context.klines_1h[market_context.klines_1h.len() - 1];
            let prev_candle = &market_context.klines_1h[market_context.klines_1h.len() - 2];

            let current_body = (current_candle.close - current_candle.open).abs();
            let prev_body = (prev_candle.close - prev_candle.open).abs();

            let is_rebound = if side == "LONG" {
                current_candle.close > current_candle.open
            } else {
                current_candle.close < current_candle.open
            };

            if is_rebound && prev_body > 0.0 && current_body > prev_body * 0.5 {
                let rebound_strength_pct = (current_body / prev_body) * 100.0;
                info!(
                    "💪 {} P1-1信号: 反弹力度{:.1}% (>50% 强支撑/护盘)",
                    symbol, rebound_strength_pct
                );
            }
        }

        let convert_to_support_klines = |source: &[Kline]| -> Vec<SupportKline> {
            source
                .iter()
                .map(|k| SupportKline {
                    open: k.open,
                    high: k.high,
                    low: k.low,
                    close: k.close,
                    volume: k.volume,
                })
                .collect()
        };

        let support_klines_5m = convert_to_support_klines(&market_context.klines_5m);
        let support_klines_15m = convert_to_support_klines(&market_context.klines_15m);
        let support_klines_1h = convert_to_support_klines(&market_context.klines_1h);

        let support_analyzer = SupportAnalyzer::new();
        let req = SupportAnalysisRequest {
            klines_5m: Some(&support_klines_5m),
            klines_15m: &support_klines_15m,
            klines_1h: &support_klines_1h,
            current_price,
            entry_price,
            sma_20: market_context.indicators.sma_20,
            sma_50: market_context.indicators.sma_50,
            bb_lower: market_context.indicators.bb_lower,
            bb_middle: market_context.indicators.bb_middle,
        };
        let support_analysis = match support_analyzer.analyze_supports(req) {
            Ok(analysis) => analysis,
            Err(e) => {
                warn!("⚠️  {} 支撑位分析失败: {}", symbol, e);
                return Ok(PositionEvaluationStep::Skip);
            }
        };
        let support_text = support_analyzer.format_support_analysis(&support_analysis);

        let last_5m_close = match market_context.klines_5m.last() {
            Some(k) => k.close,
            None => {
                warn!("⚠️  {} 5mK线数据为空", symbol);
                return Ok(PositionEvaluationStep::Skip);
            }
        };
        let deviation = ((current_price - last_5m_close) / last_5m_close) * 100.0;
        let deviation_desc = if deviation.abs() < 0.5 {
            format!("价格稳定 ({:+.2}%)", deviation)
        } else if deviation > 1.0 {
            format!("正在形成的5m K线继续上涨 {:+.2}% ✅", deviation)
        } else if deviation < -1.0 {
            format!("正在形成的5m K线继续下跌 {:+.2}% ⚠️", deviation)
        } else {
            format!("轻微波动 ({:+.2}%)", deviation)
        };

        let current_stop_loss = self
            .lookup_stop_loss(symbol, stop_loss_order_id.as_ref())
            .await;
        let current_take_profit = self
            .lookup_take_profit(symbol, take_profit_order_id.as_ref())
            .await;

        let min_notional = match min_notional_cache {
            Some(value) => value,
            None => {
                self.resolve_min_notional(symbol, &mut min_notional_cache)
                    .await?
            }
        };

        Ok(PositionEvaluationStep::Context(PreparedPositionContext {
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price,
            stop_loss_price,
            current_price,
            quantity,
            duration,
            profit_pct,
            stop_loss_order_id,
            take_profit_order_id,
            min_notional,
            market: market_context,
            support_text,
            deviation_desc,
            current_stop_loss,
            current_take_profit,
        }))
    }

    async fn collect_position_market_context(
        &self,
        symbol: &str,
    ) -> Result<Option<PositionMarketContext>> {
        fn convert_exchange_klines(raw: Vec<Vec<f64>>) -> Vec<Kline> {
            raw.into_iter()
                .map(|candle| Kline {
                    timestamp: candle.first().copied().unwrap_or_default() as i64,
                    open: candle.get(1).copied().unwrap_or_default(),
                    high: candle.get(2).copied().unwrap_or_default(),
                    low: candle.get(3).copied().unwrap_or_default(),
                    close: candle.get(4).copied().unwrap_or_default(),
                    volume: candle.get(5).copied().unwrap_or_default(),
                    quote_volume: candle.get(6).copied().unwrap_or(0.0),
                    taker_buy_volume: candle.get(7).copied().unwrap_or(0.0),
                    taker_buy_quote_volume: candle.get(8).copied().unwrap_or(0.0),
                })
                .collect()
        }

        let exchange = self.exchange.as_ref();
        let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
            time::timeout(
                time::Duration::from_secs(10),
                exchange.get_klines(symbol, "5m", Some(50)),
            ),
            time::timeout(
                time::Duration::from_secs(10),
                exchange.get_klines(symbol, "15m", Some(100)),
            ),
            time::timeout(
                time::Duration::from_secs(10),
                exchange.get_klines(symbol, "1h", Some(48)),
            ),
        );

        let klines_5m = match klines_5m_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}5mK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}5mK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        let klines_15m = match klines_15m_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}15mK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}15mK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        let klines_1h = match klines_1h_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}1hK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}1hK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        if klines_15m.len() < 20 {
            warn!(
                "⚠️  K线数据不足: {} (需要至少20根), 跳过AI评估",
                klines_15m.len()
            );
            return Ok(None);
        }

        let indicators = self.analyzer.calculate_indicators(&klines_15m);

        Ok(Some(PositionMarketContext {
            klines_5m,
            klines_15m,
            klines_1h,
            indicators,
        }))
    }

    async fn resolve_min_notional(&self, symbol: &str, cache: &mut Option<f64>) -> Result<f64> {
        if let Some(value) = cache {
            return Ok(*value);
        }

        let trading_rules = self.exchange.get_symbol_trading_rules(symbol).await?;
        let min_notional = trading_rules.min_notional.unwrap_or(5.0);
        *cache = Some(min_notional);
        Ok(min_notional)
    }

    async fn lookup_stop_loss(&self, symbol: &str, order_id: Option<&String>) -> Option<f64> {
        if let Some(order_id) = order_id {
            match self
                .exchange
                .get_order_status_detail(symbol, order_id)
                .await
            {
                Ok(order) => order.stop_price,
                Err(e) => {
                    warn!(
                        "⚠️  查询止损挂单失败: symbol={} sl_id={} err={}",
                        symbol, order_id, e
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    async fn lookup_take_profit(&self, symbol: &str, order_id: Option<&String>) -> Option<f64> {
        if let Some(order_id) = order_id {
            match self
                .exchange
                .get_order_status_detail(symbol, order_id)
                .await
            {
                Ok(order) => Some(order.price),
                Err(e) => {
                    warn!(
                        "⚠️  查询止盈挂单失败: symbol={} tp_id={} err={}",
                        symbol, order_id, e
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}
