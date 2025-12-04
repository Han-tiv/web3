use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use log::{info, warn};
use regex::Regex;
use tokio::sync::RwLock;

use crate::exchanges::binance::BinanceClient;
use crate::config::database::Database;
use crate::signals::alert_classifier::{AlertType, FundAlert};
use crate::telegram_signal::SignalAnalyzer;

/// 解析与处理消息的上下文
#[async_trait]
pub trait SignalContext: Send + Sync {
    fn exchange(&self) -> Arc<BinanceClient>;
    fn db(&self) -> &Database;
    fn tracked_coins(&self) -> Arc<RwLock<HashMap<String, FundAlert>>>;
    fn coin_ttl_hours(&self) -> i64;
    fn max_tracked_coins(&self) -> usize;
    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()>;
}

/// 消息解析器
pub struct MessageParser;

impl MessageParser {
    /// 处理来自 Telegram 的原始文本
    pub async fn handle_message<C: SignalContext>(ctx: &C, text: &str) -> Result<()> {
        if let Some(alert) = Self::parse_fund_alert(text) {
            Self::handle_incoming_alert(ctx, alert, text, true).await?;
        }
        Ok(())
    }

    /// 处理来自 Web API 的 Valuescan 信号
    pub async fn handle_valuescan_message<C: SignalContext>(
        ctx: &C,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        info!(
            "📥 处理Web信号: {} | 类型:{} | 评分:{}",
            symbol, signal_type, score
        );

        let coin = symbol.trim_end_matches("USDT").to_string();
        let exchange = ctx.exchange();

        let current_price = match exchange.get_current_price(symbol).await {
            Ok(price) => price,
            Err(e) => {
                warn!("⚠️ 获取{}当前价格失败: {}, 跳过信号", symbol, e);
                return Ok(());
            }
        };

        let alert = FundAlert {
            coin: coin.clone(),
            alert_type: AlertType::FundInflow,
            price: current_price,
            change_24h: 0.0,
            fund_type: signal_type.to_string(),
            timestamp: Utc::now(),
            raw_message: message_text.to_string(),
        };

        info!(
            "✅ Using Python parsed data: {} | coin:{} | type:{} | price:${:.4}",
            symbol, coin, signal_type, current_price
        );

        Self::handle_incoming_alert(ctx, alert, message_text, false).await
    }

    pub async fn handle_incoming_alert<C: SignalContext>(
        ctx: &C,
        mut alert: FundAlert,
        raw_message: &str,
        persist_signal: bool,
    ) -> Result<()> {
        Self::classify_alert(&mut alert);

        if persist_signal {
            Self::persist_telegram_signal(ctx, &alert, raw_message);
        }

        Self::process_classified_alert(ctx, alert).await
    }

    pub async fn process_classified_alert<C: SignalContext>(
        ctx: &C,
        alert: FundAlert,
    ) -> Result<()> {
        let signal_desc = match alert.alert_type {
            AlertType::FundEscape => "⚠️  主力出逃",
            _ => "📊 资金流入",
        };

        info!("\n{}: {} 💰", signal_desc, alert.coin);
        info!("   价格: ${:.4} | 类型: {}", alert.price, alert.fund_type);

        Self::cleanup_tracked_coins(ctx).await;

        let coins_arc = ctx.tracked_coins();
        {
            let mut coins = coins_arc.write().await;
            coins.insert(alert.coin.clone(), alert.clone());
        }

        let is_special_coin = alert.raw_message.contains("币安")
            || alert.raw_message.contains("Alpha")
            || alert.raw_message.contains("FOMO")
            || alert.raw_message.contains("出逃")
            || alert.raw_message.contains("异动");

        if !is_special_coin {
            info!(
                "⏭️ 跳过普通币种: {} (当前只交易:币安/Alpha/FOMO/出逃/异动)",
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

        info!(
            "✅ 特殊币种: {} (${:.2}), 允许交易（价格<1000）",
            alert.coin, alert.price
        );

        ctx.analyze_and_trade(alert).await
    }

    pub async fn cleanup_tracked_coins<C: SignalContext>(ctx: &C) {
        let coins_arc = ctx.tracked_coins();
        let mut coins = coins_arc.write().await;
        let now = Utc::now();

        let ttl_hours = ctx.coin_ttl_hours();
        coins.retain(|coin, alert| {
            let age_hours = (now - alert.timestamp).num_hours();
            if age_hours >= ttl_hours {
                info!("🗑️  清理过期币种: {} (已追踪 {} 小时)", coin, age_hours);
                false
            } else {
                true
            }
        });

        let max_coins = ctx.max_tracked_coins();
        if coins.len() > max_coins {
            let mut sorted: Vec<_> = coins
                .iter()
                .map(|(k, v)| (k.clone(), v.timestamp))
                .collect();
            sorted.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = coins.len() - max_coins;
            let coins_to_remove: Vec<String> = sorted
                .iter()
                .take(to_remove)
                .map(|(coin, _)| coin.clone())
                .collect();

            for coin in coins_to_remove {
                if coins.remove(&coin).is_some() {
                    info!("🗑️  容量限制,移除最旧币种: {}", coin);
                }
            }
        }
    }

    pub fn parse_fund_alert(text: &str) -> Option<FundAlert> {
        let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
        let coin = coin_regex.captures(text)?.get(1)?.as_str().to_string();

        let alert_type = if text.contains("出逃") || text.contains("撤离") {
            AlertType::FundEscape
        } else if text.contains("【资金异动】")
            || text.contains("【Alpha】")
            || text.contains("【FOMO】")
        {
            AlertType::FundInflow
        } else {
            return None;
        };

        let price_regex = Regex::new(r"现价[:：]\s*\$?([\d.]+)").ok()?;
        let price: f64 = price_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

        let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
        let change_24h: f64 = change_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

        let fund_type = if text.contains("合约") {
            "合约".to_string()
        } else if text.contains("现货") {
            "现货".to_string()
        } else {
            "未知".to_string()
        };

        Some(FundAlert {
            coin,
            alert_type,
            price,
            change_24h,
            fund_type,
            timestamp: Utc::now(),
            raw_message: text.to_string(),
        })
    }

    fn classify_alert(alert: &mut FundAlert) {
        if alert.alert_type != AlertType::FundEscape {
            alert.alert_type = AlertType::FundInflow;
        }
    }

    fn persist_telegram_signal<C: SignalContext>(ctx: &C, alert: &FundAlert, raw_message: &str) {
        let symbol = format!("{}USDT", alert.coin);
        let analyzer = SignalAnalyzer::new();
        if let Some(signal) = analyzer.analyze_message(symbol, raw_message) {
            info!(
                "📡 Telegram信号: {} 评分:{} 类型:{}",
                signal.symbol, signal.score, signal.signal_type
            );

            if let Err(err) = ctx.db().insert_telegram_signal(
                &signal.symbol,
                &signal.raw_message,
                &signal.timestamp.to_rfc3339(),
            ) {
                warn!("⚠️  保存Telegram信号失败: {}", err);
            }
        }
    }
}
