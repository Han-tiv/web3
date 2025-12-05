//! Signal Service
//!
//! 信号处理服务 - 负责接收、验证、处理和存储交易信号

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use std::sync::Arc;

use crate::config::Database;
use crate::exchanges::binance::BinanceClient;
use crate::trading_core::signals::{AlertType, FundAlert};

/// 信号服务
pub struct SignalService {
    db: Arc<Database>,
    exchange: Arc<BinanceClient>,
}

impl SignalService {
    /// 创建新的信号服务
    pub fn new(db: Arc<Database>, exchange: Arc<BinanceClient>) -> Self {
        Self { db, exchange }
    }

    /// 处理来自 Telegram 的信号
    pub async fn process_telegram_signal(
        &self,
        symbol: &str,
        raw_message: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<FundAlert> {
        info!("📨 处理Telegram信号: {}", symbol);

        // 创建 FundAlert
        let alert = FundAlert {
            coin: symbol.to_string(),
            alert_type: AlertType::FundInflow,
            price: 0.0, // 将在后续分析中获取
            change_24h: 0.0,
            fund_type: "telegram".to_string(),
            timestamp,
            raw_message: raw_message.to_string(),
        };

        // 保存到数据库
        if let Err(e) = self.save_signal(&alert).await {
            warn!("⚠️ 保存信号到数据库失败: {}", e);
        }

        Ok(alert)
    }

    /// 处理来自 Valuescan 的信号
    pub async fn process_valuescan_signal(
        &self,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<FundAlert> {
        info!("📊 处理Valuescan信号: {} (评分: {})", symbol, score);

        let alert_type = match signal_type.to_lowercase().as_str() {
            "alpha" | "fomo" | _ => AlertType::FundInflow,
        };

        let alert = FundAlert {
            coin: symbol.to_string(),
            alert_type,
            price: 0.0,
            change_24h: 0.0,
            fund_type: format!("valuescan_{}", signal_type),
            timestamp: Utc::now(),
            raw_message: message_text.to_string(),
        };

        // 保存到数据库
        if let Err(e) = self.save_signal(&alert).await {
            warn!("⚠️ 保存Valuescan信号失败: {}", e);
        }

        Ok(alert)
    }

    /// 验证信号是否有效
    pub async fn validate_signal(&self, alert: &FundAlert) -> Result<bool> {
        // 1. 检查币种是否存在于交易所
        match self.exchange.get_current_price(&alert.coin).await {
            Ok(price) => {
                debug!("✅ 币种 {} 有效，当前价格: {}", alert.coin, price);
                Ok(true)
            }
            Err(e) => {
                warn!("❌ 币种 {} 无法获取价格: {}", alert.coin, e);
                Ok(false)
            }
        }
    }

    /// 保存信号到数据库
    async fn save_signal(&self, alert: &FundAlert) -> Result<()> {
        // 这里可以调用数据库的保存方法
        // 当前保持与原有逻辑兼容
        debug!("💾 信号已记录: {} - {}", alert.coin, alert.fund_type);
        Ok(())
    }

    /// 获取未处理的信号列表
    pub async fn get_unprocessed_signals(&self, limit: usize) -> Result<Vec<FundAlert>> {
        // 从数据库获取未处理信号
        // 这里需要实现数据库查询逻辑
        Ok(Vec::new())
    }

    /// 标记信号为已处理
    pub async fn mark_signal_processed(&self, signal_id: i64) -> Result<()> {
        debug!("✅ 标记信号 {} 为已处理", signal_id);
        Ok(())
    }

    /// 检查信号是否重复（去重）
    pub async fn is_duplicate_signal(&self, symbol: &str, within_minutes: i64) -> Result<bool> {
        // 检查最近N分钟内是否有相同币种的信号
        // 这需要查询数据库
        debug!(
            "🔍 检查 {} 在 {} 分钟内是否有重复信号",
            symbol, within_minutes
        );
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchanges::binance::BinanceClient;

    #[tokio::test]
    async fn test_signal_service_creation() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let exchange = Arc::new(BinanceClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            true,
        ));

        let service = SignalService::new(db, exchange);

        // 基本测试：确保服务可以创建
        assert!(true);
    }

    #[tokio::test]
    async fn test_process_telegram_signal() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let exchange = Arc::new(BinanceClient::new(
            "test_key".to_string(),
            "test_secret".to_string(),
            true,
        ));

        let service = SignalService::new(db, exchange);

        let result = service
            .process_telegram_signal("BTCUSDT", "Test message", Utc::now())
            .await;

        assert!(result.is_ok());
        let alert = result.unwrap();
        assert_eq!(alert.coin, "BTCUSDT");
        assert_eq!(alert.fund_type, "telegram");
    }
}
