use anyhow::Result;
use reqwest;
use serde_json::json;

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self { bot_token, chat_id }
    }

    pub async fn send_trade_notification(&self, message: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let payload = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        let response = reqwest::Client::new()
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("✅ Telegram通知发送成功");
        } else {
            log::error!("❌ Telegram通知发送失败: {}", response.status());
        }

        Ok(())
    }

    pub async fn send_open_long_notification(
        &self,
        symbol: &str,
        quantity: f64,
        price: f64,
        leverage: u32,
        margin: f64,
    ) -> Result<()> {
        let message = format!(
            "🟢 <b>开多仓通知</b>\n\n\
             💰 交易对: <code>{}</code>\n\
             📈 方向: <b>做多 (LONG)</b>\n\
             🔢 数量: <code>{:.4}</code>\n\
             💵 价格: <code>{:.2} USDT</code>\n\
             ⚡ 杠杆: <code>{}x</code>\n\
             💳 保证金: <code>{:.2} USDT</code>\n\
             🕐 时间: {}",
            symbol,
            quantity,
            price,
            leverage,
            margin,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_trade_notification(&message).await
    }

    pub async fn send_open_short_notification(
        &self,
        symbol: &str,
        quantity: f64,
        price: f64,
        leverage: u32,
        margin: f64,
    ) -> Result<()> {
        let message = format!(
            "🔴 <b>开空仓通知</b>\n\n\
             💰 交易对: <code>{}</code>\n\
             📉 方向: <b>做空 (SHORT)</b>\n\
             🔢 数量: <code>{:.4}</code>\n\
             💵 价格: <code>{:.2} USDT</code>\n\
             ⚡ 杠杆: <code>{}x</code>\n\
             💳 保证金: <code>{:.2} USDT</code>\n\
             🕐 时间: {}",
            symbol,
            quantity,
            price,
            leverage,
            margin,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_trade_notification(&message).await
    }

    pub async fn send_close_position_notification(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
    ) -> Result<()> {
        let emoji = if side == "LONG" { "🟢" } else { "🔴" };
        let direction = if side == "LONG" { "多仓" } else { "空仓" };

        let message = format!(
            "{} <b>平仓通知</b>\n\n\
             💰 交易对: <code>{}</code>\n\
             📊 方向: <b>平{}</b>\n\
             🔢 数量: <code>{:.4}</code>\n\
             💵 价格: <code>{:.2} USDT</code>\n\
             🕐 时间: {}",
            emoji,
            symbol,
            direction,
            quantity,
            price,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_trade_notification(&message).await
    }

    pub async fn send_stop_loss_notification(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        loss_percent: f64,
    ) -> Result<()> {
        let message = format!(
            "🚨 <b>止损触发</b>\n\n\
             💰 交易对: <code>{}</code>\n\
             📊 方向: <code>{}</code>\n\
             🔢 数量: <code>{:.4}</code>\n\
             📉 亏损: <code>{:.1}%</code>\n\
             ⚠️ 已自动平仓保护资金\n\
             🕐 时间: {}",
            symbol,
            side,
            quantity,
            loss_percent,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_trade_notification(&message).await
    }
}
