use std::sync::Arc;

use anyhow::Result;
use log::info;
use rust_trading_bot::signals::{FundAlert, MessageParser, SignalContext};

/// 统一的信号处理器，将 Alpha/FOMO 识别以及 Telegram/Web 信号入口集中在一起。
pub struct SignalProcessor {
    alpha_keywords: Vec<String>,
    fomo_keywords: Vec<String>,
    message_parser: Arc<MessageParser>,
    context: Arc<dyn SignalContext + Send + Sync>,
}

impl SignalProcessor {
    pub fn new(
        alpha_keywords: Vec<String>,
        fomo_keywords: Vec<String>,
        message_parser: Arc<MessageParser>,
        context: Arc<dyn SignalContext + Send + Sync>,
    ) -> Self {
        Self {
            alpha_keywords,
            fomo_keywords,
            message_parser,
            context,
        }
    }

    /// 解析资金异动消息并识别 Alpha/FOMO 信号
    pub fn is_alpha_or_fomo(&self, alert: &FundAlert) -> bool {
        let message_lower = alert.raw_message.to_lowercase();

        let is_alpha = self
            .alpha_keywords
            .iter()
            .any(|kw| message_lower.contains(kw));

        let is_fomo = self
            .fomo_keywords
            .iter()
            .any(|kw| message_lower.contains(kw))
            || alert.change_24h > 10.0;

        is_alpha || is_fomo
    }

    /// 处理来自 Telegram 的原始文本
    pub async fn handle_message(&self, text: &str) -> Result<()> {
        self.message_parser
            .handle_message(self.context.as_ref(), text)
            .await
    }

    /// 处理来自 Web API 的 Valuescan 信号
    pub async fn handle_valuescan_message(
        &self,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        self.message_parser
            .handle_valuescan_message(
                self.context.as_ref(),
                symbol,
                message_text,
                score,
                signal_type,
            )
            .await
    }

    /// 处理分类后的资金异动告警
    pub async fn handle_incoming_alert(
        &self,
        alert: FundAlert,
        raw_message: &str,
        persist_signal: bool,
    ) -> Result<()> {
        self.message_parser
            .handle_incoming_alert(self.context.as_ref(), alert, raw_message, persist_signal)
            .await
    }

    /// 对 MessageParser 分类后的结果执行交易分析
    pub async fn process_classified_alert(&self, alert: FundAlert) -> Result<()> {
        if !self.is_alpha_or_fomo(&alert) {
            info!(
                "⏭️ 跳过普通币种: {} (仅交易 Alpha/FOMO/资金出逃异动)",
                alert.coin
            );
            return Ok(());
        }

        if alert.price >= 1000.0 {
            info!(
                "⏭️ 跳过高价币种: {} (${:.2}), 价格>=1000",
                alert.coin, alert.price
            );
            return Ok(());
        }

        self.message_parser
            .process_classified_alert(self.context.as_ref(), alert)
            .await
    }
}
