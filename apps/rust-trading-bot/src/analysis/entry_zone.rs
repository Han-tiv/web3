/// 入场区分析模块 - 1h主入场区 + 15m辅助入场区
///
/// 核心功能:
/// 1. 分析1h K线找到主力成本区(主入场区)
/// 2. 分析15m K线找到精细支撑位(辅助入场区)
/// 3. 综合决策最佳入场时机和仓位
use crate::analysis::market_data::Kline;
use anyhow::Result;
use log::info;

/// 置信度等级
#[derive(Debug, Clone, PartialEq)]
pub enum Confidence {
    High,   // 1h主入场区
    Medium, // 15m辅助入场区
    Low,    // 其他情况
}

/// 入场区间关系
#[derive(Debug, Clone, PartialEq)]
pub enum EntryZoneRelationship {
    Inside1H, // 15m在1h内,完美共振
    Above1H,  // 15m在1h上方,备选方案
    Below1H,  // 15m在1h下方,可能新支撑
}

/// 入场区分析结果
#[derive(Debug, Clone)]
pub struct EntryZone {
    pub ideal_entry: f64,                            // 理想入场价
    pub entry_range: (f64, f64),                     // 入场区间(下沿, 上沿)
    pub stop_loss: f64,                              // 止损价
    pub confidence: Confidence,                      // 置信度
    pub suggested_position: f64,                     // 建议仓位(0.15-0.30)
    pub relationship: Option<EntryZoneRelationship>, // 与1h的关系(仅15m有)
}

/// 入场决策
#[derive(Debug, Clone)]
pub struct EntryDecision {
    pub action: EntryAction, // 操作类型
    pub price: f64,          // 建议入场价
    pub position: f64,       // 建议仓位(0-0.30)
    pub stop_loss: f64,      // 止损价
    pub reason: String,      // 决策理由
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryAction {
    EnterNow,         // 立即入场
    EnterWithCaution, // 谨慎入场(降低仓位)
    WaitForPullback,  // 等待回调
    Skip,             // 放弃本次机会
}

/// 入场区分析器
pub struct EntryZoneAnalyzer {
    // 1h主入场区参数
    pub hourly_lookback: usize,           // 8 (分析最近8根1h)
    pub hourly_shadow_min_pct: f64,       // 2.0% (1h长下影线阈值)
    pub hourly_shadow_cluster_min: usize, // 3 (至少3根)
    pub hourly_platform_tolerance: f64,   // 0.5% (横盘容差)

    // 15m辅助入场区参数
    pub m15_lookback: usize,           // 30 (分析最近30根15m)
    pub m15_shadow_min_pct: f64,       // 1.5% (15m长下影线阈值)
    pub m15_shadow_cluster_min: usize, // 3 (至少3根)
    pub m15_platform_tolerance: f64,   // 0.5% (横盘容差)
    pub m15_platform_min_bars: usize,  // 3 (至少3根横盘)

    // 动态仓位分配
    pub position_1h_inside: f64,       // 0.30 (1h区内+15m共振)
    pub position_15m_backup: f64,      // 0.20 (15m备选入场)
    pub position_15m_new_support: f64, // 0.15 (1h破位+15m新支撑)
}

impl Default for EntryZoneAnalyzer {
    fn default() -> Self {
        Self {
            hourly_lookback: 8,
            hourly_shadow_min_pct: 2.0,
            hourly_shadow_cluster_min: 3,
            hourly_platform_tolerance: 0.5,

            m15_lookback: 30,
            m15_shadow_min_pct: 1.5,
            m15_shadow_cluster_min: 3,
            m15_platform_tolerance: 0.5,
            m15_platform_min_bars: 3,

            position_1h_inside: 0.30,
            position_15m_backup: 0.20,
            position_15m_new_support: 0.15,
        }
    }
}

impl EntryZoneAnalyzer {
    /// 分析1h K线 → 主入场区
    pub fn analyze_1h_entry_zone(&self, klines_1h: &[Kline]) -> Result<EntryZone> {
        let recent = klines_1h
            .iter()
            .rev()
            .take(self.hourly_lookback)
            .cloned()
            .collect::<Vec<_>>();

        if recent.len() < 5 {
            anyhow::bail!("1h K线数据不足,至少需要5根");
        }

        // 1. 找到价格区间
        let lowest = recent.iter().map(|k| k.low).fold(f64::INFINITY, f64::min);
        let highest = recent
            .iter()
            .map(|k| k.high)
            .fold(f64::NEG_INFINITY, f64::max);

        info!("📊 1h价格区间: ${:.4} - ${:.4}", lowest, highest);

        // 2. 识别长下影线集中区
        let shadow_zones = self.find_shadow_cluster(&recent, self.hourly_shadow_min_pct)?;

        // 3. 识别平台支撑位
        let platform_support =
            self.find_platform_support(&recent, self.hourly_platform_tolerance)?;

        // 4. 综合计算主入场区
        let entry_low = shadow_zones.0.min(platform_support.0);
        let entry_high = shadow_zones.1.max(platform_support.1);
        let ideal_entry = (entry_low + entry_high) / 2.0;

        // 5. 计算止损(最低点-1.5%)
        let stop_loss = lowest * 0.985;

        info!(
            "✅ 1h主入场区: ${:.4} - ${:.4}, 理想入场: ${:.4}, 止损: ${:.4}",
            entry_low, entry_high, ideal_entry, stop_loss
        );

        Ok(EntryZone {
            ideal_entry,
            entry_range: (entry_low, entry_high),
            stop_loss,
            confidence: Confidence::High,
            suggested_position: self.position_1h_inside,
            relationship: None,
        })
    }

    /// 分析15m K线 → 辅助入场区
    pub fn analyze_15m_entry_zone(
        &self,
        klines_15m: &[Kline],
        zone_1h: &EntryZone,
    ) -> Result<EntryZone> {
        let recent = klines_15m
            .iter()
            .rev()
            .take(self.m15_lookback)
            .cloned()
            .collect::<Vec<_>>();

        if recent.len() < 10 {
            anyhow::bail!("15m K线数据不足,至少需要10根");
        }

        // 1. 找到价格区间
        let lowest = recent.iter().map(|k| k.low).fold(f64::INFINITY, f64::min);
        let highest = recent
            .iter()
            .map(|k| k.high)
            .fold(f64::NEG_INFINITY, f64::max);

        info!("📊 15m价格区间: ${:.4} - ${:.4}", lowest, highest);

        // 2. 识别15m平台支撑
        let platform_zones = self.find_platform_support(&recent, self.m15_platform_tolerance)?;

        // 3. 识别15m下影线集中区
        let shadow_zones = self.find_shadow_cluster(&recent, self.m15_shadow_min_pct)?;

        // 4. 计算15m MA支撑
        let sma_20 = self.calculate_sma(&recent, 20);
        let sma_50 = if recent.len() >= 50 {
            self.calculate_sma(&recent, 50)
        } else {
            sma_20
        };

        // 5. 综合计算15m入场区
        let entry_low = shadow_zones.0.min(platform_zones.0).min(sma_50 * 0.995);
        let entry_high = shadow_zones.1.max(platform_zones.1).max(sma_20 * 1.005);
        let ideal_entry = (entry_low + entry_high) / 2.0;

        // 6. 判断与1h的关系
        let relationship =
            if ideal_entry >= zone_1h.entry_range.0 && ideal_entry <= zone_1h.entry_range.1 {
                EntryZoneRelationship::Inside1H
            } else if ideal_entry > zone_1h.entry_range.1 {
                EntryZoneRelationship::Above1H
            } else {
                EntryZoneRelationship::Below1H
            };

        // 7. 计算止损(15m最低点-1.5%)
        let stop_loss = lowest * 0.985;

        info!(
            "✅ 15m辅助入场区: ${:.4} - ${:.4}, 理想入场: ${:.4}, 关系: {:?}",
            entry_low, entry_high, ideal_entry, relationship
        );

        // 8. 根据关系确定建议仓位
        let suggested_position = match relationship {
            EntryZoneRelationship::Inside1H => self.position_1h_inside,
            EntryZoneRelationship::Above1H => self.position_15m_backup,
            EntryZoneRelationship::Below1H => self.position_15m_new_support,
        };

        Ok(EntryZone {
            ideal_entry,
            entry_range: (entry_low, entry_high),
            stop_loss,
            confidence: Confidence::Medium,
            suggested_position,
            relationship: Some(relationship),
        })
    }

    /// 综合决策入场策略
    pub fn decide_entry_strategy(
        &self,
        zone_1h: &EntryZone,
        zone_15m: &EntryZone,
        current_price: f64,
    ) -> EntryDecision {
        info!(
            "🤔 综合决策: 当前价=${:.4}, 1h区=[{:.4},{:.4}], 15m区=[{:.4},{:.4}]",
            current_price,
            zone_1h.entry_range.0,
            zone_1h.entry_range.1,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1
        );

        // 情况1: 当前价在1h主入场区内
        if current_price >= zone_1h.entry_range.0 && current_price <= zone_1h.entry_range.1 {
            if zone_15m.relationship == Some(EntryZoneRelationship::Inside1H) {
                // 1h+15m共振,优先在15m区间下沿入场
                let entry_price = zone_15m.entry_range.0.max(current_price * 0.998);
                return EntryDecision {
                    action: EntryAction::EnterNow,
                    price: entry_price,
                    position: self.position_1h_inside,
                    stop_loss: zone_1h.stop_loss,
                    reason: format!(
                        "✅ 1h主区内+15m共振,立即建仓 @ ${:.4} (15m下沿优化)",
                        entry_price
                    ),
                };
            } else {
                // 1h区内但15m不共振,使用1h区间
                return EntryDecision {
                    action: EntryAction::EnterNow,
                    price: current_price,
                    position: self.position_1h_inside,
                    stop_loss: zone_1h.stop_loss,
                    reason: format!("✅ 1h主区内,立即建仓 @ ${:.4}", current_price),
                };
            }
        }

        // 情况2: 当前价在1h主入场区上方
        if current_price > zone_1h.entry_range.1 {
            if zone_15m.relationship == Some(EntryZoneRelationship::Above1H)
                && current_price >= zone_15m.entry_range.0
                && current_price <= zone_15m.entry_range.1
            {
                // 15m在1h上方提供备选入场点
                return EntryDecision {
                    action: EntryAction::EnterWithCaution,
                    price: zone_15m.entry_range.0,
                    position: self.position_15m_backup,
                    stop_loss: zone_15m.stop_loss,
                    reason: format!(
                        "⚠️ 1h主区上方,15m备选入场 @ ${:.4} (仓位降至{:.0}%)",
                        zone_15m.entry_range.0,
                        self.position_15m_backup * 100.0
                    ),
                };
            } else {
                // 等待回调到1h主区
                return EntryDecision {
                    action: EntryAction::WaitForPullback,
                    price: zone_1h.entry_range.1,
                    position: self.position_1h_inside,
                    stop_loss: zone_1h.stop_loss,
                    reason: format!(
                        "⏳ 等待回调到1h主区 @ ${:.4} (挂限价单)",
                        zone_1h.entry_range.1
                    ),
                };
            }
        }

        // 情况3: 当前价在1h主入场区下方
        if current_price < zone_1h.entry_range.0 {
            if zone_15m.relationship == Some(EntryZoneRelationship::Below1H)
                && current_price >= zone_15m.entry_range.0
                && current_price <= zone_15m.entry_range.1
            {
                // 15m形成新支撑,谨慎试探
                return EntryDecision {
                    action: EntryAction::EnterWithCaution,
                    price: zone_15m.entry_range.1,
                    position: self.position_15m_new_support,
                    stop_loss: zone_15m.stop_loss,
                    reason: format!(
                        "⚠️ 1h破位,15m新支撑试探 @ ${:.4} (仓位降至{:.0}%)",
                        zone_15m.entry_range.1,
                        self.position_15m_new_support * 100.0
                    ),
                };
            } else {
                // 1h破位且15m无支撑,放弃
                return EntryDecision {
                    action: EntryAction::Skip,
                    price: 0.0,
                    position: 0.0,
                    stop_loss: 0.0,
                    reason: "❌ 1h破位且15m无明确支撑,放弃本次机会".to_string(),
                };
            }
        }

        // 默认: 跳过
        EntryDecision {
            action: EntryAction::Skip,
            price: 0.0,
            position: 0.0,
            stop_loss: 0.0,
            reason: "❌ 无法确定入场策略,放弃".to_string(),
        }
    }

    // ==================== 私有辅助方法 ====================

    /// 识别下影线集中区
    fn find_shadow_cluster(&self, klines: &[Kline], min_shadow_pct: f64) -> Result<(f64, f64)> {
        let mut shadow_lows = Vec::new();

        for k in klines {
            let lower = k.open.min(k.close);
            let shadow_pct = ((lower - k.low) / k.low) * 100.0;

            if shadow_pct >= min_shadow_pct {
                shadow_lows.push(k.low);
            }
        }

        if shadow_lows.len() < self.hourly_shadow_cluster_min {
            // 没有足够的长下影线,使用所有K线的低点
            let all_lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
            let min_low = all_lows.iter().cloned().fold(f64::INFINITY, f64::min);
            let _max_low = all_lows.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            return Ok((min_low, min_low * 1.005)); // 默认范围0.5%
        }

        let min_shadow = shadow_lows.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_shadow = shadow_lows
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        Ok((min_shadow, max_shadow))
    }

    /// 识别平台支撑位
    fn find_platform_support(&self, klines: &[Kline], tolerance_pct: f64) -> Result<(f64, f64)> {
        let mut best_platform = (0.0, 0.0);
        let mut max_count = 0;

        // 遍历每个K线作为潜在平台中心
        for i in 0..klines.len() {
            let center = (klines[i].low + klines[i].high) / 2.0;
            let mut count = 0;
            let mut lows = Vec::new();
            let mut highs = Vec::new();

            // 计算在容差范围内的K线数量
            for k in klines {
                let k_center = (k.low + k.high) / 2.0;
                let diff_pct = ((k_center - center).abs() / center) * 100.0;

                if diff_pct <= tolerance_pct {
                    count += 1;
                    lows.push(k.low);
                    highs.push(k.high);
                }
            }

            // 更新最佳平台(至少2根K线)
            if count >= 2 && count > max_count {
                max_count = count;
                let min_low = lows.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_high = highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                best_platform = (min_low, max_high);
            }
        }

        if max_count < 2 {
            // 没有找到平台,使用所有K线的范围
            let all_lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
            let all_highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
            let min_low = all_lows.iter().cloned().fold(f64::INFINITY, f64::min);
            let _max_high = all_highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            return Ok((min_low, min_low * 1.01)); // 默认范围1%
        }

        Ok(best_platform)
    }

    /// 计算简单移动平均
    fn calculate_sma(&self, klines: &[Kline], period: usize) -> f64 {
        if klines.len() < period {
            return klines.iter().map(|k| k.close).sum::<f64>() / klines.len() as f64;
        }

        let sum: f64 = klines.iter().take(period).map(|k| k.close).sum();
        sum / period as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_kline(low: f64, high: f64, open: f64, close: f64) -> Kline {
        Kline {
            timestamp: 0,
            open,
            high,
            low,
            close,
            volume: 1000.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_1h_entry_zone_analysis() {
        let analyzer = EntryZoneAnalyzer::default();

        // 构造8根1h K线,下影线集中在0.47-0.485
        let klines = vec![
            create_test_kline(0.470, 0.500, 0.475, 0.495), // 长下影线
            create_test_kline(0.475, 0.490, 0.480, 0.485),
            create_test_kline(0.472, 0.495, 0.478, 0.490), // 长下影线
            create_test_kline(0.480, 0.500, 0.485, 0.498),
            create_test_kline(0.478, 0.492, 0.482, 0.488),
            create_test_kline(0.474, 0.488, 0.480, 0.485), // 长下影线
            create_test_kline(0.485, 0.500, 0.488, 0.495),
            create_test_kline(0.480, 0.495, 0.485, 0.490),
        ];

        let zone = analyzer.analyze_1h_entry_zone(&klines).unwrap();

        assert!(zone.entry_range.0 >= 0.470 && zone.entry_range.0 <= 0.480);
        assert!(zone.entry_range.1 >= 0.485 && zone.entry_range.1 <= 0.500);
        assert_eq!(zone.confidence, Confidence::High);
    }

    #[test]
    fn test_15m_entry_zone_inside_1h() {
        let analyzer = EntryZoneAnalyzer::default();

        let klines_1h = vec![
            create_test_kline(0.470, 0.500, 0.475, 0.495),
            create_test_kline(0.475, 0.490, 0.480, 0.485),
            create_test_kline(0.472, 0.495, 0.478, 0.490),
            create_test_kline(0.480, 0.500, 0.485, 0.498),
            create_test_kline(0.478, 0.492, 0.482, 0.488),
        ];

        let zone_1h = analyzer.analyze_1h_entry_zone(&klines_1h).unwrap();

        // 构造15m K线,集中在1h区间内
        let mut klines_15m = Vec::new();
        for _ in 0..30 {
            klines_15m.push(create_test_kline(0.478, 0.487, 0.480, 0.485));
        }

        let zone_15m = analyzer
            .analyze_15m_entry_zone(&klines_15m, &zone_1h)
            .unwrap();

        assert_eq!(zone_15m.relationship, Some(EntryZoneRelationship::Inside1H));
        assert_eq!(zone_15m.confidence, Confidence::Medium);
    }
}
