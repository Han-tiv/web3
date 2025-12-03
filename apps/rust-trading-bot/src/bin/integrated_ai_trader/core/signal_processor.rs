use std::sync::Arc;

use anyhow::Result;
use rust_trading_bot::signals::{FundAlert, MessageParser, SignalContext};

/// 统一的信号处理器，负责 Alpha/FOMO 判定与消息分发。
pub struct SignalProcessor {
    alpha_keywords: Vec<String>,
    fomo_keywords: Vec<String>,
    message_parser: Arc<MessageParser>,
}

impl SignalProcessor {
    /// 创建信号处理器。
    pub fn new(
        alpha_keywords: Vec<String>,
        fomo_keywords: Vec<String>,
        message_parser: Arc<MessageParser>,
    ) -> Self {
        Self {
            alpha_keywords,
            fomo_keywords,
            message_parser,
        }
    }

    /// 返回底层的消息解析器，供需要直接访问解析器的模块复用。
    pub fn message_parser(&self) -> Arc<MessageParser> {
        self.message_parser.clone()
    }

    /// 检查一条资金异动是否满足 Alpha/FOMO 条件。
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

    /// 处理来自 Telegram 的原始消息。
    pub async fn handle_message<C: SignalContext>(&self, ctx: &C, text: &str) -> Result<()> {
        MessageParser::handle_message(ctx, text).await
    }

    /// 处理来自 HTTP/Webhook 的 Valuescan 信号。
    pub async fn handle_valuescan_message<C: SignalContext>(
        &self,
        ctx: &C,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        MessageParser::handle_valuescan_message(ctx, symbol, message_text, score, signal_type).await
    }

    /// 处理资金异动告警。
    pub async fn handle_incoming_alert<C: SignalContext>(
        &self,
        ctx: &C,
        alert: FundAlert,
        raw_message: &str,
        persist_signal: bool,
    ) -> Result<()> {
        MessageParser::handle_incoming_alert(ctx, alert, raw_message, persist_signal).await
    }

    /// 处理分类后的告警。
    pub async fn process_classified_alert<C: SignalContext>(
        &self,
        ctx: &C,
        alert: FundAlert,
    ) -> Result<()> {
        MessageParser::process_classified_alert(ctx, alert).await
    }

    /// 清理过期的追踪币种。
    pub async fn cleanup_tracked_coins<C: SignalContext>(&self, ctx: &C) {
        MessageParser::cleanup_tracked_coins(ctx).await;
    }
}
