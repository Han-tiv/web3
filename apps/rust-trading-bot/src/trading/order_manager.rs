use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use anyhow::Result;
use log::{info, warn};
use tokio::time::{sleep, Duration};

use crate::binance_client::{BinanceClient, OrderStatus};

/// 订单管理配置
#[derive(Debug, Clone)]
pub struct OrderManagerConfig {
    /// 等待限价单成交的超时时间（秒）
    pub limit_order_timeout_secs: u64,
    /// 查询限价单状态的轮询间隔（秒）
    pub poll_interval_secs: u64,
}

impl Default for OrderManagerConfig {
    fn default() -> Self {
        Self {
            limit_order_timeout_secs: 45,
            poll_interval_secs: 2,
        }
    }
}

/// 交易所订单管理器，负责限价单确认、保护单设置与取消逻辑
pub struct OrderManager {
    exchange: Arc<BinanceClient>,
    config: OrderManagerConfig,
}

impl OrderManager {
    pub fn new(exchange: Arc<BinanceClient>) -> Self {
        Self::with_config(exchange, OrderManagerConfig::default())
    }

    pub fn with_config(exchange: Arc<BinanceClient>, config: OrderManagerConfig) -> Self {
        Self { exchange, config }
    }

    /// 为组合订单等待限价单成交或部分成交，获取真实成交数量
    #[allow(dead_code)]
    pub async fn wait_for_limit_order_execution(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<Option<OrderStatus>> {
        let timeout = StdDuration::from_secs(self.config.limit_order_timeout_secs);
        let poll_interval = Duration::from_secs(self.config.poll_interval_secs);
        let start = Instant::now();
        let mut latest_status: Option<OrderStatus> = None;
        let mut last_filled_status: Option<OrderStatus> = None;
        let mut timed_out = false;

        loop {
            if start.elapsed() >= timeout {
                timed_out = true;
                break;
            }

            match self
                .exchange
                .get_order_status_detail(symbol, order_id)
                .await
            {
                Ok(status) => {
                    let state_upper = status.status.to_ascii_uppercase();
                    if status.executed_qty > f64::EPSILON {
                        last_filled_status = Some(status.clone());
                    }
                    let is_terminal = matches!(
                        state_upper.as_str(),
                        "FILLED" | "CANCELED" | "REJECTED" | "EXPIRED"
                    );
                    latest_status = Some(status.clone());

                    if is_terminal {
                        break;
                    }

                    // 已出现部分成交即可终止等待，尽快为已成交部分补上保护单
                    if status.executed_qty > f64::EPSILON {
                        break;
                    }
                }
                Err(err) => {
                    warn!(
                        "⚠️ 查询限价单状态失败 (symbol={}, order_id={}): {}",
                        symbol, order_id, err
                    );
                }
            }

            sleep(poll_interval).await;
        }

        if timed_out {
            warn!(
                "⚠️ 等待限价单成交超时 (symbol={}, order_id={}, timeout={}s)",
                symbol, order_id, self.config.limit_order_timeout_secs
            );
        }

        Ok(last_filled_status.or(latest_status))
    }

    /// 按成交数量一次性设置止损与止盈触发单
    pub async fn place_protection_orders(
        &self,
        symbol: &str,
        position_side: &str,
        quantity: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<Vec<String>> {
        if quantity <= f64::EPSILON {
            warn!(
                "⚠️ 保护单数量过小，跳过下单 (symbol={}, position_side={}, qty={:.6})",
                symbol, position_side, quantity
            );
            return Ok(Vec::new());
        }

        let mut attachments = Vec::new();

        if let Some(stop_price) = stop_loss {
            let order_id = self
                .exchange
                .set_stop_loss(symbol, position_side, quantity, stop_price, None)
                .await?;
            info!(
                "🛡️ 已设置止损: {} {} qty={:.6} stop={:.4} (order_id={})",
                symbol, position_side, quantity, stop_price, order_id
            );
            attachments.push(format!("SL {:.4}#{}", stop_price, order_id));
        }

        if let Some(take_price) = take_profit {
            let order_id = self
                .exchange
                .set_limit_take_profit(symbol, position_side, quantity, take_price)
                .await?;
            info!(
                "🎯 已设置止盈: {} {} qty={:.6} tp={:.4} (order_id={})",
                symbol, position_side, quantity, take_price, order_id
            );
            attachments.push(format!("TP {:.4}#{}", take_price, order_id));
        }

        Ok(attachments)
    }

    /// 取消单个订单
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        self.exchange.cancel_order(symbol, order_id).await
    }

    /// 批量取消订单
    pub async fn cancel_orders_batch<I, S>(&self, symbol: &str, order_ids: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for order_id in order_ids {
            self.cancel_order(symbol, order_id.as_ref()).await?;
        }
        Ok(())
    }
}
