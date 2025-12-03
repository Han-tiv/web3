use anyhow::{anyhow, Result};
use log::info;
use rust_trading_bot::{
    deepseek_client::Kline,
    entry_zone_analyzer::{EntryDecision, EntryZone, EntryZoneAnalyzer},
};
use std::sync::Arc;

pub struct EntryAnalyzer {
    entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
}

impl EntryAnalyzer {
    pub fn new(entry_zone_analyzer: Arc<EntryZoneAnalyzer>) -> Self {
        Self {
            entry_zone_analyzer,
        }
    }

    /// 分析1h和15m入场区并生成综合决策
    pub async fn analyze_entry_zones(
        &self,
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        current_price: f64,
    ) -> Result<(EntryZone, EntryZone, EntryDecision)> {
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第1步: 分析1h主入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_1h = match self.entry_zone_analyzer.analyze_1h_entry_zone(klines_1h) {
            Ok(zone) => zone,
            Err(e) => {
                return Err(anyhow!("1h entry zone analysis failed: {}", e));
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

        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第2步: 分析15m辅助入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_15m = match self
            .entry_zone_analyzer
            .analyze_15m_entry_zone(klines_15m, &zone_1h)
        {
            Ok(zone) => zone,
            Err(e) => {
                return Err(anyhow!("15m entry zone analysis failed: {}", e));
            }
        };

        info!(
            "✅ 15m辅助区: 理想价格=${:.4}, 范围=${:.4}-${:.4}, 关系={:?}",
            zone_15m.ideal_entry,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1,
            zone_15m.relationship
        );

        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🎯 第3步: 综合决策入场策略");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

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

        Ok((zone_1h, zone_15m, entry_decision))
    }
}
