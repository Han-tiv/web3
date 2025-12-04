//! Execution Service
//!
//! 执行服务 - 负责订单执行、仓位管理

use anyhow::Result;
use log::info;
use std::sync::Arc;

use crate::exchanges::binance::BinanceClient;
use crate::exchanges::{OrderResult, Position};

/// 执行服务
pub struct ExecutionService {
    exchange: Arc<BinanceClient>,
}

impl ExecutionService {
    /// 创建新的执行服务
    pub fn new(exchange: Arc<BinanceClient>) -> Self {
        Self { exchange }
    }

    /// 执行开仓
    pub async fn execute_entry(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        leverage: u32,
    ) -> Result<OrderResult> {
        info!(
            "📈 执行开仓: {} {} qty={} leverage={}x",
            side, symbol, quantity, leverage
        );

        // TODO: 设置杠杆功能需要在BinanceClient中实现
        // if let Err(e) = self.exchange.set_leverage(symbol, leverage).await {
        //     warn!("⚠️ 设置杠杆失败: {}", e);
        // }

        // 执行开仓 - 直接使用BinanceClient返回的OrderResult
        let result = if side == "LONG" {
            self.exchange.open_long(symbol, quantity, leverage, "CROSSED", false).await?
        } else {
            self.exchange.open_short(symbol, quantity, leverage, "CROSSED", false).await?
        };

        info!("✅ 开仓成功: {:?}", result);
        Ok(result)
    }

    /// 平仓
    pub async fn close_position(&self, symbol: &str, side: &str, quantity: f64) -> Result<OrderResult> {
        info!("📤 执行平仓: {} {} {}", symbol, side, quantity);

        let result = self.exchange.close_position(symbol, side, quantity).await?;

        info!("✅ 平仓成功: {:?}", result);
        Ok(result)
    }

    /// 获取所有持仓
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        use crate::exchanges::ExchangeClient; // Import trait
        self.exchange.get_positions().await
    }

    /// 获取当前价格
    pub async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        self.exchange.get_current_price(symbol).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_service_creation() {
        let exchange = Arc::new(BinanceClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            true,
        ));

        let _service = ExecutionService::new(exchange);
        assert!(true);
    }
}
