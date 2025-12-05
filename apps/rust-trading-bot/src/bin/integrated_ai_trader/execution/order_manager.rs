use anyhow::Result;
use log::{info, warn};
use rust_trading_bot::BinanceClient;
use std::sync::Arc;

/// 订单生命周期管理封装
///
/// 集中处理止损/止盈保护单以及批量取消逻辑，避免在各执行模块
/// 中重复直接调用交易所客户端。
pub struct OrderManager {
    exchange: Arc<BinanceClient>,
}

impl OrderManager {
    pub fn new(exchange: Arc<BinanceClient>) -> Self {
        Self { exchange }
    }

    /// 设置保护订单（止损 + 可选止盈）
    pub async fn place_protection_orders(
        &self,
        symbol: &str,
        position_side: &str,
        quantity: f64,
        stop_loss_price: Option<f64>,
        take_profit_price: Option<f64>,
    ) -> Result<Vec<String>> {
        info!(
            "🛡️ 设置保护单: {} side={} qty={:.6} SL={:?} TP={:?}",
            symbol, position_side, quantity, stop_loss_price, take_profit_price
        );

        let mut attachments = Vec::new();

        if let Some(sl_price) = stop_loss_price {
            let stop_loss_id = self
                .exchange
                .set_stop_loss(symbol, position_side, quantity, sl_price, None)
                .await?;
            attachments.push(format!("SL#{}", stop_loss_id));
        }

        if let Some(tp_price) = take_profit_price {
            let take_profit_id = self
                .exchange
                .set_take_profit(symbol, position_side, quantity, tp_price, None)
                .await?;
            attachments.push(format!("TP#{}", take_profit_id));
        }

        Ok(attachments)
    }

    /// 取消单个订单
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        info!("🗑️ 取消订单: {}#{}", symbol, order_id);
        self.exchange.cancel_order(symbol, order_id).await
    }

    /// 批量取消订单，若全部成功则返回 Ok
    pub async fn cancel_orders_batch(&self, symbol: &str, order_ids: &[String]) -> Result<()> {
        if order_ids.is_empty() {
            return Ok(());
        }

        let mut first_error: Option<anyhow::Error> = None;

        for order_id in order_ids {
            match self.exchange.cancel_order(symbol, order_id).await {
                Ok(_) => {
                    info!("🧹 批量取消成功: {}#{}", symbol, order_id);
                }
                Err(err) => {
                    warn!("⚠️ 批量取消失败: {}#{} - {}", symbol, order_id, err);
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if let Some(err) = first_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    /// 暴露底层 Binance 客户端，便于特殊场景复用
    pub fn exchange(&self) -> Arc<BinanceClient> {
        self.exchange.clone()
    }
}
