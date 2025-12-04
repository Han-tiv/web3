use crate::analysis::market_data::Kline;
use log::info;

/// 关键价格位类型
#[derive(Debug, Clone, PartialEq)]
pub enum LevelType {
    Support,    // 支撑位
    Resistance, // 阻力位
    Warning,    // 警戒位（可能破位）
}

/// 关键价格位
#[derive(Debug, Clone)]
pub struct KeyLevel {
    pub price: f64,
    pub level_type: LevelType,
    pub strength: f64,             // 强度评分 0-100
    pub volume: f64,               // 该位置的成交量
    pub last_test_time: i64,       // 最后一次测试时间
    pub test_count: u32,           // 被测试次数
    pub source_kline_index: usize, // 来源K线索引
}

/// 关键位识别器
pub struct KeyLevelFinder {
    price_tolerance: f64, // 价格容差百分比 (默认 0.5%)
}

impl KeyLevelFinder {
    pub fn new() -> Self {
        Self {
            price_tolerance: 0.005, // 0.5%
        }
    }

    /// 找到最近N根K线中成交量最大的K线
    pub fn find_max_volume_kline<'a>(
        &self,
        klines: &'a [Kline],
        lookback: usize,
    ) -> Option<(usize, &'a Kline)> {
        if klines.is_empty() {
            return None;
        }

        let start_idx = if klines.len() > lookback {
            klines.len() - lookback
        } else {
            0
        };

        klines[start_idx..]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.volume.partial_cmp(&b.1.volume).unwrap())
            .map(|(idx, kline)| (start_idx + idx, kline))
    }

    /// 识别所有关键位（基于最大成交量K线）
    pub fn identify_key_levels(&self, klines: &[Kline], lookback: usize) -> Vec<KeyLevel> {
        let mut levels = Vec::new();

        if let Some((idx, max_vol_kline)) = self.find_max_volume_kline(klines, lookback) {
            info!("🔍 最大成交量K线: index={}, volume={:.2}, open={:.2}, close={:.2}, high={:.2}, low={:.2}",
                idx, max_vol_kline.volume, max_vol_kline.open, max_vol_kline.close,
                max_vol_kline.high, max_vol_kline.low
            );

            let is_bullish = max_vol_kline.close > max_vol_kline.open;

            // 主力位：根据K线方向确定
            if is_bullish {
                // 大阳线：最高价为阻力位
                let resistance = KeyLevel {
                    price: max_vol_kline.high,
                    level_type: LevelType::Resistance,
                    strength: 80.0,
                    volume: max_vol_kline.volume,
                    last_test_time: max_vol_kline.timestamp,
                    test_count: 1,
                    source_kline_index: idx,
                };
                levels.push(resistance);

                // 开盘价为支撑位
                let support = KeyLevel {
                    price: max_vol_kline.open,
                    level_type: LevelType::Support,
                    strength: 70.0,
                    volume: max_vol_kline.volume,
                    last_test_time: max_vol_kline.timestamp,
                    test_count: 1,
                    source_kline_index: idx,
                };
                levels.push(support);
            } else {
                // 大阴线：最低价为支撑位
                let support = KeyLevel {
                    price: max_vol_kline.low,
                    level_type: LevelType::Support,
                    strength: 75.0,
                    volume: max_vol_kline.volume,
                    last_test_time: max_vol_kline.timestamp,
                    test_count: 1,
                    source_kline_index: idx,
                };
                levels.push(support);

                // 开盘价为阻力位
                let resistance = KeyLevel {
                    price: max_vol_kline.open,
                    level_type: LevelType::Resistance,
                    strength: 65.0,
                    volume: max_vol_kline.volume,
                    last_test_time: max_vol_kline.timestamp,
                    test_count: 1,
                    source_kline_index: idx,
                };
                levels.push(resistance);
            }

            // 增强：统计该位置被测试的次数
            self.enhance_levels_with_tests(&mut levels, klines, idx);
        }

        // 添加传统支撑阻力位
        self.add_traditional_levels(&mut levels, klines, lookback);

        levels
    }

    /// 计算关键位被测试的次数（增强强度）
    fn enhance_levels_with_tests(
        &self,
        levels: &mut [KeyLevel],
        klines: &[Kline],
        max_vol_idx: usize,
    ) {
        for level in levels.iter_mut() {
            let mut test_count = 0;
            let mut last_test_time = level.last_test_time;

            // 检查主力K线之后的K线
            for (_idx, kline) in klines.iter().enumerate().skip(max_vol_idx + 1) {
                if self.price_touches_level(kline, level.price) {
                    test_count += 1;
                    last_test_time = kline.timestamp;
                }
            }

            level.test_count += test_count;
            level.last_test_time = last_test_time;

            // 根据测试次数增强强度
            level.strength += (test_count as f64 * 5.0).min(20.0);
            level.strength = level.strength.min(100.0);
        }
    }

    /// 添加传统支撑阻力位（最近N根K线的高低点）
    fn add_traditional_levels(
        &self,
        levels: &mut Vec<KeyLevel>,
        klines: &[Kline],
        lookback: usize,
    ) {
        let start_idx = if klines.len() > lookback {
            klines.len() - lookback
        } else {
            0
        };

        let recent_klines = &klines[start_idx..];

        // 找最高点
        if let Some((idx, high_kline)) = recent_klines
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.high.partial_cmp(&b.1.high).unwrap())
        {
            // 检查是否已存在相近的阻力位
            if !levels
                .iter()
                .any(|l| self.is_same_level(l.price, high_kline.high))
            {
                levels.push(KeyLevel {
                    price: high_kline.high,
                    level_type: LevelType::Resistance,
                    strength: 60.0,
                    volume: high_kline.volume,
                    last_test_time: high_kline.timestamp,
                    test_count: 1,
                    source_kline_index: start_idx + idx,
                });
            }
        }

        // 找最低点
        if let Some((idx, low_kline)) = recent_klines
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.low.partial_cmp(&b.1.low).unwrap())
        {
            // 检查是否已存在相近的支撑位
            if !levels
                .iter()
                .any(|l| self.is_same_level(l.price, low_kline.low))
            {
                levels.push(KeyLevel {
                    price: low_kline.low,
                    level_type: LevelType::Support,
                    strength: 60.0,
                    volume: low_kline.volume,
                    last_test_time: low_kline.timestamp,
                    test_count: 1,
                    source_kline_index: start_idx + idx,
                });
            }
        }
    }

    /// 判断价格是否触及某个关键位
    fn price_touches_level(&self, kline: &Kline, level: f64) -> bool {
        let tolerance = level * self.price_tolerance;
        let in_range = kline.low <= level + tolerance && kline.high >= level - tolerance;

        // 额外检查：K线是否在该位置有明显反应（长上/下影线）
        if in_range {
            let body_size = (kline.close - kline.open).abs();
            let upper_shadow = kline.high - kline.close.max(kline.open);
            let lower_shadow = kline.open.min(kline.close) - kline.low;

            // 如果上下影线明显大于实体，说明有反应
            upper_shadow > body_size * 1.5 || lower_shadow > body_size * 1.5
        } else {
            false
        }
    }

    /// 判断两个价格是否表示同一个关键位
    fn is_same_level(&self, price1: f64, price2: f64) -> bool {
        let diff = (price1 - price2).abs();
        let tolerance = price1 * self.price_tolerance * 2.0; // 使用2倍容差
        diff < tolerance
    }

    /// 根据当前价格筛选最相关的关键位
    pub fn filter_relevant_levels(
        &self,
        levels: &[KeyLevel],
        current_price: f64,
        max_count: usize,
    ) -> Vec<KeyLevel> {
        let mut sorted_levels = levels.to_vec();

        // 按照与当前价格的距离和强度排序
        sorted_levels.sort_by(|a, b| {
            let dist_a = (a.price - current_price).abs();
            let dist_b = (b.price - current_price).abs();

            // 距离权重 70%，强度权重 30%
            let score_a = (dist_a / current_price) * 0.7 - (a.strength / 100.0) * 0.3;
            let score_b = (dist_b / current_price) * 0.7 - (b.strength / 100.0) * 0.3;

            score_a.partial_cmp(&score_b).unwrap()
        });

        sorted_levels.truncate(max_count);
        sorted_levels
    }

    /// 找到最近的支撑位和阻力位
    pub fn find_nearest_levels(
        &self,
        levels: &[KeyLevel],
        current_price: f64,
    ) -> (Option<KeyLevel>, Option<KeyLevel>) {
        let mut nearest_support: Option<KeyLevel> = None;
        let mut nearest_resistance: Option<KeyLevel> = None;

        for level in levels {
            match level.level_type {
                LevelType::Support if level.price < current_price => {
                    if let Some(ref support) = nearest_support {
                        if level.price > support.price {
                            nearest_support = Some(level.clone());
                        }
                    } else {
                        nearest_support = Some(level.clone());
                    }
                }
                LevelType::Resistance if level.price > current_price => {
                    if let Some(ref resistance) = nearest_resistance {
                        if level.price < resistance.price {
                            nearest_resistance = Some(level.clone());
                        }
                    } else {
                        nearest_resistance = Some(level.clone());
                    }
                }
                _ => {}
            }
        }

        (nearest_support, nearest_resistance)
    }

    /// 评估价格是否突破了关键位
    pub fn check_breakout(
        &self,
        current_price: f64,
        current_volume: f64,
        level: &KeyLevel,
        avg_volume: f64,
    ) -> bool {
        let price_breakout = match level.level_type {
            LevelType::Resistance => current_price > level.price * 1.002, // 突破阻力位需超过0.2%
            LevelType::Support => current_price < level.price * 0.998,    // 跌破支撑位需低于0.2%
            _ => false,
        };

        // 成交量确认：需要大于平均成交量的1.5倍
        let volume_confirm = current_volume > avg_volume * 1.5;

        price_breakout && volume_confirm
    }

    /// 格式化关键位信息
    pub fn format_levels(&self, levels: &[KeyLevel]) -> String {
        let mut result = String::from("【关键价格位】\n");

        for (i, level) in levels.iter().enumerate() {
            let type_str = match level.level_type {
                LevelType::Support => "支撑",
                LevelType::Resistance => "阻力",
                LevelType::Warning => "警戒",
            };

            result.push_str(&format!(
                "{}. {} ${:.2} | 强度:{:.0}% | 测试:{}次\n",
                i + 1,
                type_str,
                level.price,
                level.strength,
                level.test_count
            ));
        }

        result
    }

    /// 基于净流入识别主力关键位
    ///
    /// # 参数
    /// - klines: 1h K线数据(必须包含净流入字段)
    /// - symbol: 交易对名称
    /// - lookback_hours: 回溯小时数 (默认24)
    ///
    /// # 返回
    /// 返回按净流入排序的关键位列表,最多5个
    pub fn identify_inflow_key_levels(
        &self,
        klines: &[Kline],
        symbol: &str,
        lookback_hours: usize,
    ) -> Vec<KeyLevel> {
        if klines.is_empty() {
            return Vec::new();
        }

        // 1. 确定净流入阈值
        let threshold = if symbol == "BTCUSDT" || symbol == "ETHUSDT" {
            100_000_000.0 // 1亿 USDT
        } else {
            5_000_000.0 // 500万 USDT
        };

        // 2. 确定回溯范围
        let start_idx = if klines.len() > lookback_hours {
            klines.len() - lookback_hours
        } else {
            0
        };

        // 3. 筛选满足净流入阈值的K线
        let mut candidates: Vec<(usize, &Kline, f64)> = klines[start_idx..]
            .iter()
            .enumerate()
            .filter_map(|(idx, kline)| {
                let net_inflow = kline.volume; // TODO: taker_buy_quote_volume field does not exist in Kline struct
                if net_inflow >= threshold {
                    Some((start_idx + idx, kline, net_inflow))
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            info!(
                "⚠️ {} 最近{}小时内无净流入 ≥ {:.0}万 的K线",
                symbol,
                lookback_hours,
                threshold / 10_000.0
            );
            return Vec::new();
        }

        // 4. 按净流入从大到小排序
        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // 5. 取前5个净流入最大的K线
        candidates.truncate(5);

        // 6. 为每个候选K线创建关键位
        let mut levels = Vec::new();
        for (idx, kline, net_inflow) in candidates {
            // 中间价格 = (最高价 + 最低价) / 2
            let mid_price = (kline.high + kline.low) / 2.0;

            // 强度: 根据净流入金额计算 (归一化到 60-100)
            let strength = 60.0 + (net_inflow / threshold * 40.0).min(40.0);

            // 类型判断: 中间价高于收盘价视为阻力,低于收盘价视为支撑
            let level_type = if mid_price > kline.close {
                LevelType::Resistance
            } else {
                LevelType::Support
            };

            let type_str = match &level_type {
                LevelType::Support => "支撑",
                LevelType::Resistance => "阻力",
                _ => "未知",
            };

            levels.push(KeyLevel {
                price: mid_price,
                level_type,
                strength,
                volume: kline.volume,
                last_test_time: kline.timestamp,
                test_count: 1,
                source_kline_index: idx,
            });

            info!(
                "🎯 主力关键位: {} ${:.2} ({}) | 净流入: {:.2}万 USDT",
                type_str,
                mid_price,
                symbol,
                net_inflow / 10_000.0
            );
        }

        levels
    }
}

impl Default for KeyLevelFinder {
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
    fn test_find_max_volume_kline() {
        let klines = vec![
            sample_kline(1, 100.0, 105.0, 98.0, 103.0, 1000.0),
            sample_kline(2, 103.0, 110.0, 102.0, 108.0, 5000.0),
            sample_kline(3, 108.0, 112.0, 106.0, 110.0, 2000.0),
        ];

        let finder = KeyLevelFinder::new();
        let result = finder.find_max_volume_kline(&klines, 10);

        assert!(result.is_some());
        let (idx, kline) = result.unwrap();
        assert_eq!(idx, 1);
        assert_eq!(kline.volume, 5000.0);
    }

    #[test]
    fn test_identify_key_levels() {
        let klines = vec![
            sample_kline(1, 100.0, 105.0, 98.0, 103.0, 1000.0),
            sample_kline(2, 103.0, 110.0, 102.0, 108.0, 5000.0),
            sample_kline(3, 108.0, 112.0, 106.0, 110.0, 2000.0),
        ];

        let finder = KeyLevelFinder::new();
        let levels = finder.identify_key_levels(&klines, 10);

        assert!(!levels.is_empty());

        // 应该至少有一个阻力位和一个支撑位
        let has_resistance = levels.iter().any(|l| l.level_type == LevelType::Resistance);
        let has_support = levels.iter().any(|l| l.level_type == LevelType::Support);

        assert!(has_resistance);
        assert!(has_support);
    }
}
