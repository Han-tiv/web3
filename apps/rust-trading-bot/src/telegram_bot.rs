use crate::copy_trader::{CopyTradeStats, CopyTrader};
use log::info;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    utils::command::BotCommands,
};
use tokio::sync::Mutex;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "支持的命令:")]
pub enum Command {
    #[command(description = "显示帮助信息")]
    Help,
    #[command(description = "启动跟单")]
    Start,
    #[command(description = "停止跟单")]
    Stop,
    #[command(description = "查看账户状态")]
    Status,
    #[command(description = "查看持仓")]
    Positions,
    #[command(description = "查看统计")]
    Stats,
    #[command(description = "设置跟单比例 (例如: /ratio 0.5)")]
    Ratio(String),
}

pub struct TelegramBot {
    bot: Bot,
    copy_trader: Arc<Mutex<CopyTrader>>,
    is_running: Arc<Mutex<bool>>,
}

impl TelegramBot {
    pub fn new(token: String, copy_trader: CopyTrader) -> Self {
        Self {
            bot: Bot::new(token),
            copy_trader: Arc::new(Mutex::new(copy_trader)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn run(&self) {
        info!("🤖 Telegram Bot 启动中...");

        let handler = Update::filter_message()
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(Self::handle_command),
            )
            .branch(dptree::endpoint(Self::handle_message));

        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![
                self.copy_trader.clone(),
                self.is_running.clone()
            ])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }

    async fn handle_command(
        bot: Bot,
        msg: Message,
        cmd: Command,
        copy_trader: Arc<Mutex<CopyTrader>>,
        is_running: Arc<Mutex<bool>>,
    ) -> ResponseResult<()> {
        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }

            Command::Start => {
                let mut running = is_running.lock().await;
                if *running {
                    bot.send_message(msg.chat.id, "⚠️ 跟单已经在运行中").await?;
                } else {
                    *running = true;
                    let trader = copy_trader.clone();

                    // 在后台启动跟单
                    tokio::spawn(async move {
                        if let Err(e) = trader.lock().await.start_monitoring().await {
                            log::error!("跟单监控错误: {}", e);
                        }
                    });

                    bot.send_message(msg.chat.id, "✅ 跟单已启动！")
                        .parse_mode(ParseMode::Html)
                        .await?;
                }
            }

            Command::Stop => {
                let mut running = is_running.lock().await;
                *running = false;
                bot.send_message(msg.chat.id, "⏹️ 跟单已停止").await?;
            }

            Command::Status => {
                let trader = copy_trader.lock().await;
                match trader.get_statistics().await {
                    Ok(stats) => {
                        let status_msg = Self::format_status_message(&stats);
                        bot.send_message(msg.chat.id, status_msg)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ 获取状态失败: {}", e))
                            .await?;
                    }
                }
            }

            Command::Positions => {
                let trader = copy_trader.lock().await;
                match trader.get_statistics().await {
                    Ok(stats) => {
                        let positions_msg = Self::format_positions_message(&stats);
                        bot.send_message(msg.chat.id, positions_msg)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ 获取持仓失败: {}", e))
                            .await?;
                    }
                }
            }

            Command::Stats => {
                let trader = copy_trader.lock().await;
                match trader.get_statistics().await {
                    Ok(stats) => {
                        let stats_msg = Self::format_stats_message(&stats);
                        bot.send_message(msg.chat.id, stats_msg)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ 获取统计失败: {}", e))
                            .await?;
                    }
                }
            }

            Command::Ratio(ratio_str) => match ratio_str.parse::<f64>() {
                Ok(ratio) if ratio > 0.0 && ratio <= 1.0 => {
                    bot.send_message(
                        msg.chat.id,
                        format!("✅ 跟单比例已设置为: {}%", ratio * 100.0),
                    )
                    .await?;
                }
                _ => {
                    bot.send_message(msg.chat.id, "❌ 无效的比例，请输入 0.0 到 1.0 之间的数值")
                        .await?;
                }
            },
        }

        Ok(())
    }

    async fn handle_message(bot: Bot, msg: Message) -> ResponseResult<()> {
        if let Some(text) = msg.text() {
            bot.send_message(msg.chat.id, format!("收到消息: {}", text))
                .await?;
        }
        Ok(())
    }

    fn format_status_message(stats: &CopyTradeStats) -> String {
        format!(
            "📊 <b>账户状态</b>\n\n\
             💰 总余额: <code>{:.2} USDT</code>\n\
             💵 可用余额: <code>{:.2} USDT</code>\n\
             📈 未实现盈亏: <code>{:.2} USDT</code>\n\
             📦 持仓数量: <code>{}</code>",
            stats.balance, stats.available_balance, stats.total_pnl, stats.position_count
        )
    }

    fn format_positions_message(stats: &CopyTradeStats) -> String {
        if stats.positions.is_empty() {
            return "📭 当前无持仓".to_string();
        }

        let mut msg = "📦 <b>当前持仓</b>\n\n".to_string();

        for (i, pos) in stats.positions.iter().enumerate() {
            let profit_emoji = if pos.pnl > 0.0 { "🟢" } else { "🔴" };

            msg.push_str(&format!(
                "{}. <b>{}</b> {}\n\
                 ├ 方向: <code>{}</code>\n\
                 ├ 数量: <code>{:.4}</code>\n\
                 ├ 入场价: <code>{:.2}</code>\n\
                 ├ 标记价: <code>{:.2}</code>\n\
                 ├ 杠杆: <code>{}x</code>\n\
                 └ 盈亏: <code>{:.2} USDT</code> {}\n\n",
                i + 1,
                pos.symbol,
                profit_emoji,
                pos.side,
                pos.size,
                pos.entry_price,
                pos.mark_price,
                pos.leverage,
                pos.pnl,
                profit_emoji
            ));
        }

        msg
    }

    fn format_stats_message(stats: &CopyTradeStats) -> String {
        let win_rate = if stats.position_count > 0 {
            let winning = stats.positions.iter().filter(|p| p.pnl > 0.0).count();
            (winning as f64 / stats.position_count as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "📈 <b>跟单统计</b>\n\n\
             💰 账户余额: <code>{:.2} USDT</code>\n\
             📊 总盈亏: <code>{:.2} USDT</code>\n\
             📦 持仓数: <code>{}</code>\n\
             ✅ 胜率: <code>{:.1}%</code>",
            stats.balance, stats.total_pnl, stats.position_count, win_rate
        )
    }

    pub fn create_inline_keyboard() -> InlineKeyboardMarkup {
        let buttons = vec![
            vec![
                InlineKeyboardButton::callback("▶️ 启动", "start"),
                InlineKeyboardButton::callback("⏹️ 停止", "stop"),
            ],
            vec![
                InlineKeyboardButton::callback("📊 状态", "status"),
                InlineKeyboardButton::callback("📦 持仓", "positions"),
            ],
            vec![InlineKeyboardButton::callback("📈 统计", "stats")],
        ];

        InlineKeyboardMarkup::new(buttons)
    }
}
