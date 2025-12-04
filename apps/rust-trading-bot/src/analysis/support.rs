use anyhow::Result;

/// Phase 2.4 (#13): 支撑位分析请求参数
pub struct SupportAnalysisRequest<'a> {
    pub klines_5m: Option<&'a [Kline]>,
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub current_price: f64,
    pub entry_price: f64,
    pub sma_20: f64,
    pub sma_50: f64,
    pub bb_lower: f64,
    pub bb_middle: f64,
}

#[derive(Debug, Clone)]
pub struct SupportLevel {
    pub price: f64,
    pub strength: u8, // 1-10分
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

impl Default for SupportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SupportAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 方案2: 简化版支撑位识别（3大算法）
    pub fn analyze_supports(&self, req: SupportAnalysisRequest<'_>) -> Result<SupportAnalysis> {
        // 从request解构参数
        let klines_15m = req.klines_15m;
        let klines_1h = req.klines_1h;
        let current_price = req.current_price;
        let entry_price = req.entry_price;
        let sma_20 = req.sma_20;
        let sma_50 = req.sma_50;
        let bb_lower = req.bb_lower;
        let bb_middle = req.bb_middle;

        // ========== 算法1: 下影线密集法 ==========
        let shadow_15m = self.find_shadow_cluster(klines_15m, current_price);
        let shadow_1h = self.find_shadow_cluster(klines_1h, current_price);

        // ========== 算法2: 前期平台法 ==========
        let platform_15m = self.find_platform_level(klines_15m, current_price);
        let platform_1h = self.find_platform_level(klines_1h, current_price);

        // ========== 算法3: 均线共振法 ==========
        let ma_resonance = self.find_ma_resonance(sma_20, sma_50, bb_middle, current_price);

        // ========== Level 1: 短期支撑（15m级别）- 取1个最强 ==========
        let mut level1_candidates = Vec::new();

        // BOLL下轨（动态支撑）
        level1_candidates.push(SupportLevel {
            price: bb_lower,
            strength: 6,
            source: "BOLL下轨".to_string(),
            test_count: 1,
            distance_pct: ((current_price - bb_lower) / current_price) * 100.0,
        });

        if let Some(s) = shadow_15m {
            level1_candidates.push(s);
        }
        if let Some(s) = platform_15m {
            level1_candidates.push(s);
        }

        level1_candidates.sort_by(|a, b| b.strength.cmp(&a.strength));
        let level1_supports = vec![level1_candidates.into_iter().next().unwrap()];

        // ========== Level 2: 中期支撑（1h级别）- 取1个最强 ==========
        let mut level2_candidates = Vec::new();

        // 1h SMA20
        level2_candidates.push(SupportLevel {
            price: sma_20,
            strength: 7,
            source: "1h SMA20".to_string(),
            test_count: 1,
            distance_pct: ((current_price - sma_20) / current_price) * 100.0,
        });

        if let Some(s) = shadow_1h {
            level2_candidates.push(s);
        }
        if let Some(s) = platform_1h {
            level2_candidates.push(s);
        }

        level2_candidates.sort_by(|a, b| b.strength.cmp(&a.strength));
        let level2_supports = vec![level2_candidates.into_iter().next().unwrap()];

        // ========== Level 3: 关键支撑（核心防线）- 取1个最强 ==========
        let mut level3_candidates = Vec::new();

        // 1h SMA50（重要均线）
        level3_candidates.push(SupportLevel {
            price: sma_50,
            strength: 9,
            source: "1h SMA50".to_string(),
            test_count: 1,
            distance_pct: ((current_price - sma_50) / current_price) * 100.0,
        });

        // 入场保本位
        let breakeven_price = entry_price * 0.99; // 入场价-1%
        level3_candidates.push(SupportLevel {
            price: breakeven_price,
            strength: 10,
            source: "入场保本位".to_string(),
            test_count: 1,
            distance_pct: ((current_price - breakeven_price) / current_price) * 100.0,
        });

        // 均线共振位
        if let Some(s) = ma_resonance {
            level3_candidates.push(s);
        }

        level3_candidates.sort_by(|a, b| b.strength.cmp(&a.strength));
        let level3_supports = vec![level3_candidates.into_iter().next().unwrap()];

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

    /// 算法1: 下影线密集法
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

    /// 算法2: 前期平台法
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

    /// 算法3: 均线共振法
    fn find_ma_resonance(
        &self,
        sma_20: f64,
        sma_50: f64,
        bb_middle: f64,
        current_price: f64,
    ) -> Option<SupportLevel> {
        // 如果多条均线在±1%范围内聚集，形成共振支撑
        let mas = [sma_20, sma_50, bb_middle];
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

    /// 格式化支撑位分析为文本（方案2简化版）
    pub fn format_support_analysis(&self, analysis: &SupportAnalysis) -> String {
        let mut text = String::from("【方案2: 简化版多级支撑位系统】\n");
        text.push_str("算法: 下影线密集法 + 前期平台法 + 均线共振法\n\n");

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
        text.push_str("📊 策略: 接近此区域+1m长下影线 → 第1次止盈60%\n\n");

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
        text.push_str(
            "📊 策略: 距离Level2<3%时观察，若获支撑继续持有，若下破+成交量增大→第2次止盈(全平)\n\n",
        );

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
        text.push_str("🚨 策略: 跌破Level3+无反弹+成交量放大 → 立即全部平仓\n\n");

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
