use crate::analysis::market_data::Kline;
use crate::key_level_finder::{KeyLevel, KeyLevelFinder};
use crate::technical_analysis::TechnicalAnalyzer;
use log::{info, warn};

/// Phase 2.4 (#14): 做多信号生成上下文
pub struct LongSignalContext<'a> {
    pub current_price: f64,
    pub indicators: &'a crate::analysis::market_data::TechnicalIndicators,
    pub key_levels: &'a [KeyLevel],
    pub nearest_support: Option<&'a KeyLevel>,
    pub nearest_resistance: Option<&'a KeyLevel>,
    pub money_flow_strength: f64,
    pub volume_ratio: f64,
    pub current_position: Option<&'a str>,
}

/// 主力资金流向
#[derive(Debug, Clone, PartialEq)]
pub enum MoneyFlowDirection {
    Inflow,  // 流入
    Outflow, // 流出
    Neutral, // 中性
}

/// 主力资金信号
#[derive(Debug, Clone)]
pub struct MoneyFlowSignal {
    pub timestamp: i64,
    pub direction: MoneyFlowDirection,
    pub strength: f64,  // 0.0-1.0 流向强度
    pub source: String, // 信号来源（telegram/api）
    pub symbol: String,
    pub note: Option<String>, // 备注信息
}

/// 交易信号类型
#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    LongBreakout,   // 突破做多
    LongPullback,   // 回踩做多
    ShortBreakdown, // 破位做空
    ClosePosition,  // 平仓
    Hold,           // 持有
}

/// 信号优先级
#[derive(Debug, Clone, PartialEq)]
pub enum SignalPriority {
    Critical, // 立即执行
    High,     // 高优先级
    Medium,   // 中等
    Low,      // 低优先级
}

/// 交易信号
#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub signal_type: SignalType,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub position_size: f64,
    pub priority: SignalPriority,
    pub reason: String,
    pub confidence: f64, // 0-100
    pub key_levels: Vec<KeyLevel>,
}

/// 主力资金追踪器
pub struct SmartMoneyTracker {
    level_finder: KeyLevelFinder,
    analyzer: TechnicalAnalyzer,

    // 配置参数
    lookback_hours: usize,          // 回看小时数（1h K线）
    min_money_flow_strength: f64,   // 最小资金流向强度
    min_volume_ratio: f64,          // 最小成交量比率
    key_level_score_threshold: f64, // 关键位强度阈值
}

impl SmartMoneyTracker {
    pub fn new() -> Self {
        Self {
            level_finder: KeyLevelFinder::new(),
            analyzer: TechnicalAnalyzer::new(),
            lookback_hours: 24,
            min_money_flow_strength: 0.6,
            min_volume_ratio: 1.5,
            key_level_score_threshold: 60.0,
        }
    }

    /// 分析主力资金并生成交易信号
    pub fn analyze_and_generate_signal(
        &self,
        money_flow: &MoneyFlowSignal,
        klines: &[Kline],
        current_price: f64,
        current_position: Option<&str>, // "long", "short", None
    ) -> Option<TradingSignal> {
        info!("🎯 开始分析主力资金信号");
        info!(
            "   资金方向: {:?}, 强度: {:.2}",
            money_flow.direction, money_flow.strength
        );

        // 1. 检查资金流向强度
        if money_flow.strength < self.min_money_flow_strength {
            warn!(
                "⚠️  资金流向强度不足: {:.2} < {:.2}",
                money_flow.strength, self.min_money_flow_strength
            );
            return None;
        }

        // 2. 计算技术指标
        let indicators = self.analyzer.calculate_indicators(klines);

        // 3. 识别关键价格位
        let all_levels = self
            .level_finder
            .identify_key_levels(klines, self.lookback_hours);
        let key_levels = self
            .level_finder
            .filter_relevant_levels(&all_levels, current_price, 5);

        info!("{}", self.level_finder.format_levels(&key_levels));

        // 4. 找到最近的支撑和阻力位
        let (nearest_support, nearest_resistance) = self
            .level_finder
            .find_nearest_levels(&key_levels, current_price);

        // 5. 计算平均成交量
        let avg_volume = self.calculate_avg_volume(klines, 20);
        let current_volume = klines.last().map(|k| k.volume).unwrap_or(0.0);
        let volume_ratio = current_volume / avg_volume;

        info!("   当前成交量比率: {:.2}", volume_ratio);

        // 6. 根据主力资金方向生成信号
        match money_flow.direction {
            MoneyFlowDirection::Inflow => {
                let ctx = LongSignalContext {
                    current_price,
                    indicators: &indicators,
                    key_levels: &key_levels,
                    nearest_support: nearest_support.as_ref(),
                    nearest_resistance: nearest_resistance.as_ref(),
                    money_flow_strength: money_flow.strength,
                    volume_ratio,
                    current_position,
                };
                self.generate_long_signal(ctx)
            }
            MoneyFlowDirection::Outflow => self.generate_short_or_close_signal(
                current_price,
                &indicators,
                &key_levels,
                nearest_support.as_ref(),
                money_flow.strength,
                current_position,
            ),
            MoneyFlowDirection::Neutral => None,
        }
    }

    /// 生成做多信号
    fn generate_long_signal(&self, ctx: LongSignalContext<'_>) -> Option<TradingSignal> {
        // 从context解构参数
        let current_price = ctx.current_price;
        let indicators = ctx.indicators;
        let key_levels = ctx.key_levels;
        let nearest_support = ctx.nearest_support;
        let nearest_resistance = ctx.nearest_resistance;
        let money_flow_strength = ctx.money_flow_strength;
        let volume_ratio = ctx.volume_ratio;
        let current_position = ctx.current_position;

        // 场景1：突破做多
        if let Some(resistance) = nearest_resistance {
            if current_price > resistance.price * 0.998 && volume_ratio > self.min_volume_ratio {
                return Some(self.create_breakout_long_signal(
                    current_price,
                    resistance,
                    nearest_support,
                    money_flow_strength,
                    volume_ratio,
                    key_levels,
                ));
            }
        }

        // 场景2：回踩支撑做多
        if let Some(support) = nearest_support {
            let price_near_support = (current_price - support.price).abs() / support.price < 0.01; // 1%范围内
            let rsi_oversold = indicators.rsi_15m < 40.0;

            if price_near_support
                && rsi_oversold
                && support.strength > self.key_level_score_threshold
            {
                return Some(self.create_pullback_long_signal(
                    current_price,
                    support,
                    nearest_resistance,
                    money_flow_strength,
                    indicators.rsi_15m,
                    key_levels,
                ));
            }
        }

        // 场景3：已持有多单，持续资金流入 - 持有
        if current_position == Some("long") {
            info!("✅ 已持有多单，资金持续流入，建议持有");
            return Some(TradingSignal {
                signal_type: SignalType::Hold,
                entry_price: current_price,
                stop_loss: nearest_support
                    .map(|s| s.price * 0.98)
                    .unwrap_or(current_price * 0.97),
                take_profit: nearest_resistance
                    .map(|r| r.price)
                    .unwrap_or(current_price * 1.05),
                position_size: 0.0,
                priority: SignalPriority::Low,
                reason: "资金流入持续，持有多单".to_string(),
                confidence: 70.0,
                key_levels: key_levels.to_vec(),
            });
        }

        None
    }

    /// 生成做空或平仓信号
    fn generate_short_or_close_signal(
        &self,
        current_price: f64,
        indicators: &crate::analysis::market_data::TechnicalIndicators,
        key_levels: &[KeyLevel],
        nearest_support: Option<&KeyLevel>,
        money_flow_strength: f64,
        current_position: Option<&str>,
    ) -> Option<TradingSignal> {
        // 场景1：持有多单 + 资金流出 → 平仓
        if current_position == Some("long") {
            warn!("⚠️  资金流出，建议平多仓");
            return Some(TradingSignal {
                signal_type: SignalType::ClosePosition,
                entry_price: current_price,
                stop_loss: 0.0,
                take_profit: 0.0,
                position_size: 0.0,
                priority: if money_flow_strength > 0.8 {
                    SignalPriority::Critical
                } else {
                    SignalPriority::High
                },
                reason: format!("资金大量流出(强度:{:.2})，平仓止损", money_flow_strength),
                confidence: 80.0,
                key_levels: key_levels.to_vec(),
            });
        }

        // 场景2：破位做空
        if let Some(support) = nearest_support {
            if current_price < support.price * 0.998 && indicators.rsi_15m < 35.0 {
                warn!("🔻 跌破支撑位，考虑做空");
                return Some(TradingSignal {
                    signal_type: SignalType::ShortBreakdown,
                    entry_price: current_price,
                    stop_loss: support.price * 1.02,
                    take_profit: current_price * 0.97,
                    position_size: 0.0, // 由外部仓位管理器计算
                    priority: SignalPriority::High,
                    reason: format!(
                        "跌破支撑位 ${:.2}, RSI:{:.1}",
                        support.price, indicators.rsi_15m
                    ),
                    confidence: 75.0,
                    key_levels: key_levels.to_vec(),
                });
            }
        }

        None
    }

    /// 创建突破做多信号
    fn create_breakout_long_signal(
        &self,
        current_price: f64,
        resistance: &KeyLevel,
        nearest_support: Option<&KeyLevel>,
        money_flow_strength: f64,
        volume_ratio: f64,
        key_levels: &[KeyLevel],
    ) -> TradingSignal {
        let stop_loss = nearest_support
            .map(|s| s.price * 0.98)
            .unwrap_or(current_price * 0.97);

        let take_profit = current_price * 1.05; // 5% 目标

        let confidence = 60.0 + (money_flow_strength * 20.0) + ((volume_ratio - 1.0) * 10.0);

        info!("🚀 生成突破做多信号");

        TradingSignal {
            signal_type: SignalType::LongBreakout,
            entry_price: current_price,
            stop_loss,
            take_profit,
            position_size: 0.0,
            priority: if confidence > 85.0 {
                SignalPriority::Critical
            } else {
                SignalPriority::High
            },
            reason: format!(
                "突破阻力位 ${:.2}, 资金流入强度:{:.2}, 成交量:{:.1}倍",
                resistance.price, money_flow_strength, volume_ratio
            ),
            confidence: confidence.min(100.0),
            key_levels: key_levels.to_vec(),
        }
    }

    /// 创建回踩做多信号
    fn create_pullback_long_signal(
        &self,
        current_price: f64,
        support: &KeyLevel,
        nearest_resistance: Option<&KeyLevel>,
        money_flow_strength: f64,
        rsi: f64,
        key_levels: &[KeyLevel],
    ) -> TradingSignal {
        let stop_loss = support.price * 0.98;
        let take_profit = nearest_resistance
            .map(|r| r.price * 0.99)
            .unwrap_or(current_price * 1.04);

        let confidence = 65.0 + (money_flow_strength * 15.0) + ((40.0 - rsi) * 0.5);

        info!("📈 生成回踩做多信号");

        TradingSignal {
            signal_type: SignalType::LongPullback,
            entry_price: current_price,
            stop_loss,
            take_profit,
            position_size: 0.0,
            priority: SignalPriority::Medium,
            reason: format!(
                "回踩支撑位 ${:.2}, RSI超卖:{:.1}, 资金流入:{:.2}",
                support.price, rsi, money_flow_strength
            ),
            confidence: confidence.min(100.0),
            key_levels: key_levels.to_vec(),
        }
    }

    /// 计算平均成交量
    fn calculate_avg_volume(&self, klines: &[Kline], period: usize) -> f64 {
        if klines.is_empty() {
            return 0.0;
        }

        let start = if klines.len() > period {
            klines.len() - period
        } else {
            0
        };

        let sum: f64 = klines[start..].iter().map(|k| k.volume).sum();
        sum / (klines.len() - start) as f64
    }

    /// 格式化交易信号
    pub fn format_signal(&self, signal: &TradingSignal) -> String {
        format!(
            r#"
【交易信号】
类型: {:?}
优先级: {:?}
入场价: ${:.2}
止损价: ${:.2}
止盈价: ${:.2}
置信度: {:.1}%
理由: {}
"#,
            signal.signal_type,
            signal.priority,
            signal.entry_price,
            signal.stop_loss,
            signal.take_profit,
            signal.confidence,
            signal.reason
        )
    }
}

impl Default for SmartMoneyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_kline(
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Kline {
        Kline {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            ..Default::default()
        }
    }

    #[test]
    fn test_money_flow_signal_creation() {
        let signal = MoneyFlowSignal {
            timestamp: 1234567890,
            direction: MoneyFlowDirection::Inflow,
            strength: 0.8,
            source: "telegram".to_string(),
            symbol: "BTC/USDT".to_string(),
            note: Some("主力大量买入".to_string()),
        };

        assert_eq!(signal.direction, MoneyFlowDirection::Inflow);
        assert_eq!(signal.strength, 0.8);
    }

    #[test]
    fn test_calculate_avg_volume() {
        let klines = vec![
            sample_kline(1, 100.0, 105.0, 98.0, 103.0, 1000.0),
            sample_kline(2, 103.0, 110.0, 102.0, 108.0, 2000.0),
            sample_kline(3, 108.0, 112.0, 106.0, 110.0, 3000.0),
        ];

        let tracker = SmartMoneyTracker::new();
        let avg = tracker.calculate_avg_volume(&klines, 3);

        assert_eq!(avg, 2000.0);
    }
}
