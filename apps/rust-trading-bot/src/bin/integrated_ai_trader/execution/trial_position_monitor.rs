use std::sync::Weak;

use anyhow::Result;
use log::{error, info, warn};
use rust_trading_bot::{
    deepseek_client::Kline, exchange_trait::ExchangeClient, staged_position_manager::PositionStage,
};
use tokio::time;

use super::super::trader::IntegratedAITrader;

pub struct TrialPositionMonitor {
    trader: Weak<IntegratedAITrader>,
}

impl TrialPositionMonitor {
    pub fn new(trader: Weak<IntegratedAITrader>) -> Self {
        Self { trader }
    }

    /// 监控试探持仓,检测启动信号并执行70%补仓
    pub async fn monitor(&self) -> Result<()> {
        let Some(trader) = self.trader.upgrade() else {
            warn!("⚠️ TrialPositionMonitor: 无法获取交易器实例");
            return Ok(());
        };

        let staged_manager = trader.staged_manager.read().await;
        let trial_positions: Vec<String> = staged_manager
            .positions
            .iter()
            .filter_map(|(symbol, pos)| {
                if matches!(pos.stage, PositionStage::TrialPosition) {
                    Some(symbol.clone())
                } else {
                    None
                }
            })
            .collect();
        drop(staged_manager);

        for symbol in trial_positions {
            info!("\n🔍 检查试探持仓: {}", symbol);

            // 获取多周期K线数据 (1m, 5m, 15m, 1h)
            let (klines_1m_result, klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
                time::timeout(
                    time::Duration::from_secs(10),
                    trader.exchange.get_klines(&symbol, "1m", Some(10))
                ),
                time::timeout(
                    time::Duration::from_secs(10),
                    trader.exchange.get_klines(&symbol, "5m", Some(50))
                ),
                time::timeout(
                    time::Duration::from_secs(10),
                    trader.exchange.get_klines(&symbol, "15m", Some(100))
                ),
                time::timeout(
                    time::Duration::from_secs(10),
                    trader.exchange.get_klines(&symbol, "1h", Some(48))
                )
            );

            // 解析K线数据 - 转换为Kline结构体
            let _klines_1m = match klines_1m_result {
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
                _ => {
                    warn!("⚠️  获取{}1mK线失败,跳过启动信号检测", symbol);
                    continue;
                }
            };

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
                _ => {
                    warn!("⚠️  获取{}5mK线失败,跳过启动信号检测", symbol);
                    continue;
                }
            };

            let klines_15m = match klines_15m_result {
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
                _ => {
                    warn!("⚠️  获取{}15mK线失败,跳过启动信号检测", symbol);
                    continue;
                }
            };

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
                _ => {
                    warn!("⚠️  获取{}1hK线失败,跳过启动信号检测", symbol);
                    continue;
                }
            };

            // 检测启动信号
            let staged_manager_read = trader.staged_manager.read().await;
            let position_opt = staged_manager_read.positions.get(&symbol).cloned();
            drop(staged_manager_read);

            if let Some(position) = position_opt {
                // 获取当前价格
                let current_price = match trader.exchange.get_current_price(&symbol).await {
                    Ok(price) => price,
                    Err(e) => {
                        warn!("⚠️  获取{}当前价格失败: {}", symbol, e);
                        continue;
                    }
                };

                match trader.launch_detector.detect_launch_signal(
                    &klines_5m,
                    &klines_15m,
                    &klines_1h,
                    position.trial_entry_price,
                    current_price,
                ) {
                    Ok(launch_signal) => {
                        info!(
                            "🚀 启动信号检测: 5m={} | 15m={} | 1h={} | 1m偏离={:.2}% | 全部确认={} | 得分={:.0}",
                            launch_signal.m5_signal,
                            launch_signal.m15_trend,
                            launch_signal.h1_breakout,
                            launch_signal.m1_deviation,
                            launch_signal.all_confirmed,
                            launch_signal.score
                        );
                        info!("   理由: {}", launch_signal.reason);

                        // 判断是否应该补仓
                        let staged_manager_read = trader.staged_manager.read().await;
                        let should_add = staged_manager_read
                            .should_add_position(&symbol, &launch_signal)
                            .unwrap_or(false);
                        drop(staged_manager_read);

                        if should_add {
                            info!("✅ 启动信号全部确认,准备执行70%补仓");

                            let current_price =
                                match trader.exchange.get_current_price(&symbol).await {
                                    Ok(price) => price,
                                    Err(e) => {
                                        error!("❌ 获取{}当前价格失败: {}", symbol, e);
                                        continue;
                                    }
                                };

                            let mut staged_manager = trader.staged_manager.write().await;
                            let (available_usdt, leverage) =
                                (trader.max_position_usdt, trader.max_leverage as f64);

                            match staged_manager.execute_add_position(
                                &symbol,
                                current_price,
                                available_usdt,
                                leverage,
                            ) {
                                Ok(_) => {
                                    info!("✅ 70%补仓执行成功");
                                    if let Some(snapshot) = staged_manager.positions.get(&symbol) {
                                        info!("   试探入场: ${:.4}", snapshot.trial_entry_price);
                                        info!("   补仓入场: ${:.4}", snapshot.add_entry_price);
                                        info!("   平均成本: ${:.4}", snapshot.avg_cost);
                                        info!("   总仓位: {:.6}", snapshot.total_quantity);
                                    }

                                    let mut trackers = trader.position_trackers.write().await;
                                    if let Some(tracker) = trackers.get_mut(&symbol) {
                                        let new_quantity = staged_manager
                                            .positions
                                            .get(&symbol)
                                            .map(|p| p.total_quantity)
                                            .unwrap_or(tracker.quantity);
                                        let new_entry_price = staged_manager
                                            .positions
                                            .get(&symbol)
                                            .map(|p| p.avg_cost)
                                            .unwrap_or(tracker.entry_price);

                                        tracker.quantity = new_quantity;
                                        tracker.entry_price = new_entry_price;
                                        info!(
                                            "✅ 已同步tracker: 数量{:.6} → 成本${:.4}",
                                            new_quantity, new_entry_price
                                        );
                                    }
                                    drop(trackers);
                                }
                                Err(e) => {
                                    error!("❌ 70%补仓执行失败: {}", e);
                                }
                            }
                        } else {
                            info!("⏸️  启动信号未全部确认,继续等待");
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  启动信号检测失败: {}", e);
                    }
                }
            }
        }

        Ok(())
    }
}
