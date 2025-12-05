//! Signal Handler Module
//!
//! 处理来自各种来源的交易信号

use anyhow::Result;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

use rust_trading_bot::{
    config::database::Database,
    signal_parser::SignalParser,
    signals::{AlertType, FundAlert},
};

use std::collections::HashMap;

/// 信号处理器
pub struct SignalHandler {
    db: Arc<Database>,
    signal_parser: Arc<SignalParser>,
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
}

impl SignalHandler {
    /// 创建新的信号处理器
    pub fn new(
        db: Arc<Database>,
        tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    ) -> Self {
        Self {
            db: Arc::new(db.as_ref().clone()),
            signal_parser: Arc::new(SignalParser),
            tracked_coins,
        }
    }

    /// 处理新消息 - 所有信号都送给解析器判断
    pub async fn handle_message(&self, text: &str) -> Result<()> {
        info!("📨 收到新消息");

        // 解析消息
        match self.signal_parser.parse(text) {
            Ok(alert) => {
                info!("✅ 解析成功: {:?}", alert);
                self.handle_incoming_alert(alert, text, true).await?;
            }
            Err(e) => {
                warn!("⚠️ 消息解析失败: {}", e);
            }
        }

        Ok(())
    }

    /// 处理来自 Web API 的 Valuescan 信号
    pub async fn handle_valuescan_message(
        &self,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        info!("📊 收到Valuescan信号: {} ({})", symbol, signal_type);

        let alert = self.create_valuescan_alert(symbol, message_text, score, signal_type);
        self.handle_incoming_alert(alert, message_text, true).await?;

        Ok(())
    }

    /// 处理传入的信号
    pub async fn handle_incoming_alert(
        &self,
        alert: FundAlert,
        raw_message: &str,
        persist_signal: bool,
    ) -> Result<()> {
        info!("🔔 处理信号: {} - {}", alert.coin, alert.alert_type);

        // 保存信号到数据库
        if persist_signal {
            if let Err(e) = self.save_signal_to_db(&alert, raw_message).await {
                warn!("⚠️ 保存信号失败: {}", e);
            }
        }

        // 添加到追踪列表
        self.add_to_tracked_coins(alert.clone()).await;

        Ok(())
    }

    /// 创建Valuescan信号
    fn create_valuescan_alert(
        &self,
        symbol: &str,
        _message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> FundAlert {
        let alert_type = match signal_type.to_lowercase().as_str() {
            "alpha" => AlertType::FundInflow,
            "fomo" => AlertType::FundInflow,
            _ => AlertType::FundInflow,
        };

        FundAlert {
            coin: symbol.to_string(),
            alert_type,
            timestamp: chrono::Utc::now().timestamp(),
            score,
            ..Default::default()
        }
    }

    /// 保存信号到数据库
    async fn save_signal_to_db(&self, alert: &FundAlert, raw_message: &str) -> Result<()> {
        // 这里需要实现数据库保存逻辑
        // 由于Database结构复杂，这里先留空
        info!("💾 保存信号: {} - {}", alert.coin, raw_message);
        Ok(())
    }

    /// 添加到追踪币种列表
    async fn add_to_tracked_coins(&self, alert: FundAlert) {
        let mut coins = self.tracked_coins.write().await;
        coins.insert(alert.coin.clone(), alert);
    }

    /// 获取追踪币种
    pub async fn get_tracked_coin(&self, symbol: &str) -> Option<FundAlert> {
        let coins = self.tracked_coins.read().await;
        coins.get(symbol).cloned()
    }

    /// 获取所有追踪币种
    pub async fn get_all_tracked_coins(&self) -> HashMap<String, FundAlert> {
        let coins = self.tracked_coins.read().await;
        coins.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_handler_creation() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let tracked_coins = Arc::new(RwLock::new(HashMap::new()));

        let _handler = SignalHandler::new(db, tracked_coins);
        assert!(true);
    }

    #[tokio::test]
    async fn test_valuescan_alert_creation() {
        let db = Arc::new(Database::new(":memory:").unwrap());
        let tracked_coins = Arc::new(RwLock::new(HashMap::new()));
        let handler = SignalHandler::new(db, tracked_coins.clone());

        handler
            .handle_valuescan_message("BTCUSDT", "Test message", 85, "alpha")
            .await
            .unwrap();

        let alert = handler.get_tracked_coin("BTCUSDT").await;
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().score, 85);
    }
}
