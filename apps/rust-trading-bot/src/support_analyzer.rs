use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SupportLevel {
    pub price: f64,
    pub strength: u8,         // 1-10分
    pub source: String,
    pub test_count: usize,
    pub distance_pct: f64,
}

#[derive(Debug, Clone)]
pub struct MultiLevelSupports {
    pub level1_short_term: Vec<SupportLevel>,
    pub level2_mid_term: Vec<SupportLevel>,
    pub level3_key_level: Vec<SupportLevel>,
}

#[derive(Debug, Clone)]
pub struct SupportAnalysis {
    pub supports: MultiLevelSupports,
    pub nearest_support: SupportLevel,
    pub strongest_support: SupportLevel,
    pub break_risk: String,
}

#[derive(Debug, Clone)]
pub struct Kline {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub struct SupportAnalyzer;

impl SupportAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 完整版支撑位识别（5大算法综合）
    pub fn analyze_supports(
        &self,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        current_price: f64,
        entry_price: f64,
        sma_20: f64,
        sma_50: f64,
        bb_lower: f64,
        bb_middle: f64,
    ) -> Result<SupportAnalysis> {
        // ========== Level 1: 短期支撑（15m级别）==========
        let mut level1_supports = Vec::new();

        // 1.1 BOLL下轨（动态支撑）
        level1_supports.push(SupportLevel {
            price: bb_lower,
            strength: 6,
            source: "15m BOLL下轨".to_string(),
            test_count: 1,
            distance_pct: ((current_price - bb_lower) / current_price) * 100.0,
        });

        // 1.2 15m下影线密集区
        if let Some(shadow_support) = self.find_shadow_cluster(klines_15m, current_price) {
            level1_supports.push(shadow_support);
        }

        // 1.3 15m成交量堆积区
        if let Some(volume_support) = self.find_volume_peak(klines_15m, current_price, "15m") {
            level1_supports.push(volume_support);
        }

        // ========== Level 2: 中期支撑（1h级别）==========
        let mut level2_supports = Vec::new();

        // 2.1 1h SMA20
        level2_supports.push(SupportLevel {
            price: sma_20,
            strength: 7,
            source: "1h SMA20".to_string(),
            test_count: 1,
            distance_pct: ((current_price - sma_20) / current_price) * 100.0,
        });

        // 2.2 1h前期平台位
        if let Some(platform_support) = self.find_platform_level(klines_1h, current_price) {
            level2_supports.push(platform_support);
        }

        // 2.3 1h下影线密集区
        if let Some(shadow_support) = self.find_shadow_cluster(klines_1h, current_price) {
            level2_supports.push(shadow_support);
        }

        // ========== Level 3: 关键支撑（核心防线）==========
        let mut level3_supports = Vec::new();

        // 3.1 1h SMA50（重要均线）
        level3_supports.push(SupportLevel {
            price: sma_50,
            strength: 9,
            source: "1h SMA50".to_string(),
            test_count: 1,
            distance_pct: ((current_price - sma_50) / current_price) * 100.0,
        });

        // 3.2 入场保本位
        let breakeven_price = entry_price * 0.99; // 入场价-1%
        level3_supports.push(SupportLevel {
            price: breakeven_price,
            strength: 10,
            source: "入场保本位".to_string(),
            test_count: 1,
            distance_pct: ((current_price - breakeven_price) / current_price) * 100.0,
        });

        // 3.3 1h最大成交量堆积区
        if let Some(volume_support) = self.find_volume_peak(klines_1h, current_price, "1h") {
            level3_supports.push(volume_support);
        }

        // 3.4 均线共振位
        if let Some(resonance_support) = self.find_ma_resonance(sma_20, sma_50, bb_middle, current_price) {
            level3_supports.push(resonance_support);
        }

        // 3.5 斐波那契回撤位
        if let Some(fib_support) = self.find_fibonacci_level(klines_1h, current_price) {
            level3_supports.push(fib_support);
        }

        // ========== 排序和筛选 ==========
        level1_supports.sort_by(|a, b| b.strength.cmp(&a.strength));
        level2_supports.sort_by(|a, b| b.strength.cmp(&a.strength));
        level3_supports.sort_by(|a, b| b.strength.cmp(&a.strength));

        // 保留每级前3个最强支撑
        level1_supports.truncate(3);
        level2_supports.truncate(3);
        level3_supports.truncate(3);

        // ========== 找最近和最强支撑位 ==========
        let all_supports: Vec<SupportLevel> = level1_supports
            .iter()
            .chain(level2_supports.iter())
            .chain(level3_supports.iter())
            .cloned()
            .collect();

        let nearest_support = all_supports
            .iter()
            .filter(|s| s.price < current_price)
            .min_by(|a, b| {
                let dist_a = (current_price - a.price).abs();
                let dist_b = (current_price - b.price).abs();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .cloned()
            .unwrap_or_else(|| level3_supports[0].clone());

        let strongest_support = all_supports
            .iter()
            .max_by_key(|s| s.strength)
            .cloned()
            .unwrap_or_else(|| level3_supports[0].clone());

        // ========== 计算破位风险 ==========
        let break_risk = if nearest_support.distance_pct < 1.0 {
            "高 ⚠️".to_string()
        } else if nearest_support.distance_pct < 3.0 {
            "中 📊".to_string()
        } else {
            "低 ✅".to_string()
        };

        Ok(SupportAnalysis {
            supports: MultiLevelSupports {
                level1_short_term: level1_supports,
                level2_mid_term: level2_supports,
                level3_key_level: level3_supports,
            },
            nearest_support,
            strongest_support,
            break_risk,
        })
    }

    /// 算法1: 成交量堆积法
    fn find_volume_peak(&self, klines: &[Kline], current_price: f64, timeframe: &str) -> Option<SupportLevel> {
        if klines.is_empty() {
            return None;
        }

        // 将价格按0.5%分段，统计每段的累计成交量
        let mut price_volume_map: HashMap<u32, (f64, f64)> = HashMap::new(); // (累计成交量, 平均价格)

        for kline in klines.iter().rev().take(30) {
            let price_bucket = ((kline.close / current_price * 200.0) as u32); // 0.5%分段
            let entry = price_volume_map.entry(price_bucket).or_insert((0.0, 0.0));
            entry.0 += kline.volume;
            entry.1 += kline.close;
        }

        // 找成交量最大的价格区间
        let max_entry = price_volume_map
            .iter()
            .filter(|(bucket, _)| {
                let bucket_price = (**bucket as f64) * current_price / 200.0;
                bucket_price < current_price // 只考虑当前价格下方的支撑
            })
            .max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap())?;

        let bucket_price = (*max_entry.0 as f64) * current_price / 200.0;
        let volume_strength = ((max_entry.1 .0 / klines.len() as f64).min(10.0)) as u8;

        Some(SupportLevel {
            price: bucket_price,
            strength: volume_strength.max(5),
            source: format!("{}成交量堆积区", timeframe),
            test_count: 1,
            distance_pct: ((current_price - bucket_price) / current_price) * 100.0,
        })
    }

    /// 算法2: 下影线密集法
    fn find_shadow_cluster(&self, klines: &[Kline], current_price: f64) -> Option<SupportLevel> {
        if klines.is_empty() {
            return None;
        }

        // 统计下影线最低点分布
        let shadows: Vec<f64> = klines
            .iter()
            .rev()
            .take(20)
            .filter(|k| k.low < k.open.min(k.close)) // 有下影线
            .map(|k| k.low)
            .collect();

        if shadows.len() < 3 {
            return None;
        }

        // 找下影线最低点的平均值（密集区）
        let avg_shadow_low = shadows.iter().sum::<f64>() / shadows.len() as f64;
        let test_count = shadows
            .iter()
            .filter(|&&low| (low - avg_shadow_low).abs() / avg_shadow_low < 0.01)
            .count();

        Some(SupportLevel {
            price: avg_shadow_low,
            strength: (5 + test_count.min(5)) as u8,
            source: "下影线密集区".to_string(),
            test_count,
            distance_pct: ((current_price - avg_shadow_low) / current_price) * 100.0,
        })
    }

    /// 算法3: 前期平台法
    fn find_platform_level(&self, klines: &[Kline], current_price: f64) -> Option<SupportLevel> {
        if klines.len() < 10 {
            return None;
        }

        // 找连续5根以上K线收盘价波动 < 2% 的区域
        for i in 0..klines.len().saturating_sub(5) {
            let window = &klines[i..i + 5];
            let closes: Vec<f64> = window.iter().map(|k| k.close).collect();
            let max_close = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_close = closes.iter().cloned().fold(f64::INFINITY, f64::min);

            if (max_close - min_close) / min_close < 0.02 {
                // 找到平台
                let platform_price = (max_close + min_close) / 2.0;

                if platform_price < current_price {
                    return Some(SupportLevel {
                        price: platform_price,
                        strength: 7,
                        source: "前期横盘平台".to_string(),
                        test_count: 5,
                        distance_pct: ((current_price - platform_price) / current_price) * 100.0,
                    });
                }
            }
        }

        None
    }

    /// 算法4: 均线共振法
    fn find_ma_resonance(
        &self,
        sma_20: f64,
        sma_50: f64,
        bb_middle: f64,
        current_price: f64,
    ) -> Option<SupportLevel> {
        // 如果多条均线在±1%范围内聚集，形成共振支撑
        let mas = vec![sma_20, sma_50, bb_middle];
        let avg_ma = mas.iter().sum::<f64>() / mas.len() as f64;

        let resonance_count = mas
            .iter()
            .filter(|&&ma| (ma - avg_ma).abs() / avg_ma < 0.01)
            .count();

        if resonance_count >= 2 && avg_ma < current_price {
            Some(SupportLevel {
                price: avg_ma,
                strength: (6 + resonance_count) as u8,
                source: "均线共振位".to_string(),
                test_count: resonance_count,
                distance_pct: ((current_price - avg_ma) / current_price) * 100.0,
            })
        } else {
            None
        }
    }

    /// 算法5: 斐波那契回撤法
    fn find_fibonacci_level(&self, klines: &[Kline], current_price: f64) -> Option<SupportLevel> {
        if klines.len() < 20 {
            return None;
        }

        // 找最近的波段高低点
        let recent_high = klines
            .iter()
            .rev()
            .take(20)
            .map(|k| k.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let recent_low = klines
            .iter()
            .rev()
            .take(20)
            .map(|k| k.low)
            .fold(f64::INFINITY, f64::min);

        let range = recent_high - recent_low;

        // 计算斐波那契回撤位: 0.382, 0.5, 0.618, 0.786
        let fib_levels = vec![
            ("38.2%", recent_high - range * 0.382),
            ("50%", recent_high - range * 0.5),
            ("61.8%", recent_high - range * 0.618),
            ("78.6%", recent_high - range * 0.786),
        ];

        // 找最接近当前价格下方的斐波那契位
        fib_levels
            .iter()
            .filter(|(_, price)| *price < current_price)
            .min_by(|a, b| {
                let dist_a = (current_price - a.1).abs();
                let dist_b = (current_price - b.1).abs();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(level, price)| SupportLevel {
                price: *price,
                strength: 8,
                source: format!("斐波那契{}", level),
                test_count: 1,
                distance_pct: ((current_price - price) / current_price) * 100.0,
            })
    }

    /// 格式化支撑位分析为文本
    pub fn format_support_analysis(&self, analysis: &SupportAnalysis) -> String {
        let mut text = String::from("【完整版多级支撑位系统】\n\n");

        // Level 1
        text.push_str("━━━ Level 1: 短期支撑（15m级别）━━━\n");
        text.push_str("强度评级: ★★☆☆☆ (容易突破)\n");
        for (i, support) in analysis.supports.level1_short_term.iter().enumerate() {
            text.push_str(&format!(
                "{}. {} ${:.4} (距离: {:.2}%) [强度: {}/10, 测试{}次]\n",
                i + 1,
                support.source,
                support.price,
                support.distance_pct,
                support.strength,
                support.test_count
            ));
        }
        text.push_str("📊 策略: 接近此区域+1m长下影线 → 部分止盈50%-60%\n\n");

        // Level 2
        text.push_str("━━━ Level 2: 中期支撑（1h级别）━━━\n");
        text.push_str("强度评级: ★★★☆☆ (较强支撑)\n");
        for (i, support) in analysis.supports.level2_mid_term.iter().enumerate() {
            text.push_str(&format!(
                "{}. {} ${:.4} (距离: {:.2}%) [强度: {}/10, 测试{}次]\n",
                i + 1,
                support.source,
                support.price,
                support.distance_pct,
                support.strength,
                support.test_count
            ));
        }
        text.push_str("📊 策略: 跌破Level1向Level2靠近 → 观察是否获得支撑\n\n");

        // Level 3
        text.push_str("━━━ Level 3: 关键支撑（核心防线）━━━\n");
        text.push_str("强度评级: ★★★★★ (核心支撑，破位必走)\n");
        for (i, support) in analysis.supports.level3_key_level.iter().enumerate() {
            text.push_str(&format!(
                "{}. {} ${:.4} (距离: {:.2}%) [强度: {}/10, 测试{}次]\n",
                i + 1,
                support.source,
                support.price,
                support.distance_pct,
                support.strength,
                support.test_count
            ));
        }
        text.push_str("🚨 策略: 跌破此区域+成交量放大+无反弹 → 全部平仓\n\n");

        // 关键信息
        text.push_str(&format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
            🎯 最近支撑: {} ${:.4} (距离{:.2}%)\n\
            💪 最强支撑: {} ${:.4} (强度{}/10)\n\
            ⚠️  破位风险: {}\n",
            analysis.nearest_support.source,
            analysis.nearest_support.price,
            analysis.nearest_support.distance_pct,
            analysis.strongest_support.source,
            analysis.strongest_support.price,
            analysis.strongest_support.strength,
            analysis.break_risk
        ));

        text
    }
}
