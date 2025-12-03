use std::sync::Weak;

use anyhow::Result;
use log::{debug, error, info, warn};
use rust_trading_bot::exchange_trait::Position;

use super::super::trader::IntegratedAITrader;

pub struct PositionProtector {
    trader: Weak<IntegratedAITrader>,
}

impl PositionProtector {
    pub fn new(trader: Weak<IntegratedAITrader>) -> Self {
        Self { trader }
    }

    /// 执行小仓位保护:保证金<0.5U+亏损+资金费率不利时平仓
    pub async fn execute(&self, exchange_positions: &[Position]) -> Result<()> {
        let Some(trader) = self.trader.upgrade() else {
            warn!("⚠️ PositionProtector: 无法获取交易器实例");
            return Ok(());
        };

        debug!(
            "🔍 开始小仓位保护检查: 共{}个持仓",
            exchange_positions.len()
        );
        for position in exchange_positions {
            let symbol = position.symbol.clone();

            let symbol_rules = match trader.exchange.get_symbol_trading_rules(&symbol).await {
                Ok(rules) => rules,
                Err(e) => {
                    warn!("⚠️  {} 获取交易规则失败: {}", symbol, e);
                    continue;
                }
            };

            if position.size.abs() < symbol_rules.min_qty {
                warn!(
                    "⚠️  {} 持仓数量 {:.8} 小于最小交易数量 {:.8}，无法通过 API 平仓，跳过处理",
                    symbol,
                    position.size.abs(),
                    symbol_rules.min_qty
                );
                let mut trackers = trader.position_trackers.write().await;
                trackers.remove(&symbol);
                info!("✅ {} 已从追踪器中移除（尘埃持仓）", symbol);
                continue;
            }

            debug!("   检查持仓: {} size={:.8}", symbol, position.size);
            if position.size <= f64::EPSILON {
                debug!("   {} 仓位过小,跳过", symbol);
                continue;
            }

            let is_long = if position.side.eq_ignore_ascii_case("LONG") {
                true
            } else if position.side.eq_ignore_ascii_case("SHORT") {
                false
            } else {
                warn!(
                    "⚠️ {} 未知持仓方向({}), 跳过单仓保护",
                    symbol,
                    position.side.as_str()
                );
                continue;
            };
            let signed_size = if is_long {
                position.size
            } else {
                -position.size
            };

            if position.entry_price <= 0.0 {
                warn!(
                    "⚠️ {} 入场价异常({:.4}), 跳过单仓保护",
                    symbol, position.entry_price
                );
                continue;
            }

            let notional = signed_size.abs() * position.entry_price;
            let margin = notional / 15.0;

            debug!(
                "   {} 保证金计算: notional=${:.2}, margin=${:.4}",
                symbol, notional, margin
            );

            if margin >= 0.5 {
                debug!("   {} 保证金{:.4}U >= 0.5U,不触发保护", symbol, margin);
                continue;
            }

            info!("✅ {} 符合保证金条件: {:.4}U < 0.5U", symbol, margin);

            let should_close = if position.pnl < 0.0 {
                info!(
                    "🚨 {} 小仓位亏损保护触发: 保证金{:.2}U, PnL={:.4}, 方向={}",
                    symbol,
                    margin,
                    position.pnl,
                    position.side.as_str()
                );
                true
            } else {
                let funding_rate = match trader.exchange.get_funding_rate(&symbol).await {
                    Ok((rate, _, _, _, _)) => rate,
                    Err(e) => {
                        warn!("⚠️ {} 获取资金费率失败: {}", symbol, e);
                        continue;
                    }
                };

                let unfavorable_funding =
                    (is_long && funding_rate > 0.0) || (!is_long && funding_rate < 0.0);

                if unfavorable_funding {
                    info!(
                        "🚨 {} 小仓位盈利+资金费率不利保护触发: 保证金{:.2}U, PnL={:.4}, 资金费率={:.4}%, 方向={}",
                        symbol,
                        margin,
                        position.pnl,
                        funding_rate * 100.0,
                        position.side.as_str()
                    );
                }

                unfavorable_funding
            };

            if !should_close {
                continue;
            }

            error!(
                "🚨 {} 执行小仓位保护平仓: 保证金{:.2}U, PnL={:.4}, 方向={}",
                symbol,
                margin,
                position.pnl,
                position.side.as_str()
            );

            match trader.close_position_fully_with_retry(&symbol, 3).await {
                Ok(_) => {
                    trader
                        .send_critical_alert(&symbol, "小仓位亏损+资金费率不利,执行保护平仓...")
                        .await;
                }
                Err(e) => {
                    error!("❌ {} 小仓位亏损保护平仓失败: {}", symbol, e);
                    trader
                        .send_critical_alert(&symbol, "小仓位亏损+资金费率不利,执行保护平仓...")
                        .await;
                }
            }
        }

        Ok(())
    }
}
