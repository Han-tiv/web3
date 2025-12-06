// 订单管理器模块
use anyhow::Result;
use log::{info, warn};
use std::sync::Arc;

use crate::binance_client::BinanceClient;

/// 订单管理器
pub struct OrderManager {
    exchange: Arc<BinanceClient>,
}

impl OrderManager {
    pub fn new(exchange: Arc<BinanceClient>) -> Self {
        Self { exchange }
    }
    
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        self.exchange.cancel_order(symbol, order_id).await
    }
    
    /// 批量取消订单
    pub async fn cancel_orders_batch(&self, symbol: &str, order_ids: &[String]) -> Result<()> {
        for order_id in order_ids {
            if let Err(e) = self.exchange.cancel_order(symbol, order_id).await {
                warn!("⚠️ 取消订单{}失败: {}", order_id, e);
            }
        }
        Ok(())
    }
    
    /// 设置止损止盈保护单
    pub async fn place_protection_orders(
        &self,
        symbol: &str,
        side: &str,      // "LONG" 或 "SHORT"
        quantity: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<(Option<String>, Option<String>)> {
        let mut sl_order_id = None;
        let mut tp_order_id = None;
        
        // 设置止损
        if let Some(sl_price) = stop_loss {
            match self.exchange.place_trigger_order(
                symbol,
                "STOP_MARKET",  // 触发单类型
                "CLOSE",        // 平仓动作
                side,           // LONG/SHORT
                quantity,
                sl_price,
                None,           // 市价单不需要limit_price
            ).await {
                Ok(order_id) => {
                    info!("✅ 止损单已设: {} @ {:.4}", symbol, sl_price);
                    sl_order_id = Some(order_id);
                }
                Err(e) => {
                    warn!("⚠️ 止损单失败: {}", e);
                }
            }
        }
        
        // 设置止盈
        if let Some(tp_price) = take_profit {
            match self.exchange.place_trigger_order(
                symbol,
                "TAKE_PROFIT_MARKET",  // 触发单类型
                "CLOSE",               // 平仓动作
                side,                  // LONG/SHORT
                quantity,
                tp_price,
                None,                  // 市价单不需要limit_price
            ).await {
                Ok(order_id) => {
                    info!("✅ 止盈单已设: {} @ {:.4}", symbol, tp_price);
                    tp_order_id = Some(order_id);
                }
                Err(e) => {
                    warn!("⚠️ 止盈单失败: {}", e);
                }
            }
        }
        
        Ok((sl_order_id, tp_order_id))
    }
}
