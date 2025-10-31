use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{Client, Config, InitParams};
use grammers_session::Session;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserMessage {
    message_id: i32,
    timestamp: String,
    content: String,
    has_media: bool,
    is_reply: bool,
    reply_to_msg_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserHistoryReport {
    channel_name: String,
    channel_id: i64,
    user_id: i64,
    username: String,
    total_messages: usize,
    date_range: String,
    messages: Vec<UserMessage>,
    statistics: MessageStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageStatistics {
    total_count: usize,
    with_media: usize,
    replies: usize,
    avg_message_length: f64,
    keywords_found: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("═══════════════════════════════════════════════════");
    println!("📜 Telegram 用户历史记录整理工具");
    println!("═══════════════════════════════════════════════════\n");

    // 目标配置
    let channel_id = 2488739133_i64; // 目标群组
    let target_user_id = 2069693449_i64; // 目标用户
    
    println!("🎯 目标配置:");
    println!("  频道: valuescan (ID: {})", channel_id);
    println!("  用户 ID: {}", target_user_id);
    println!();

    // 连接到 Telegram
    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;

    println!("🔄 连接到 Telegram...");

    let client = Client::connect(Config {
        session: Session::load_file_or_create("session.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: InitParams {
            device_model: "Desktop".to_string(),
            system_version: "Windows 10".to_string(),
            app_version: "5.12.3 x64".to_string(),
            lang_code: "en".to_string(),
            system_lang_code: "en-US".to_string(),
            catch_up: true,
            ..Default::default()
        },
    })
    .await?;

    if !client.is_authorized().await? {
        println!("❌ 未登录，请先运行 list_channels");
        return Ok(());
    }

    println!("✅ 已连接\n");

    // 查找频道
    println!("🔍 查找频道...");
    let mut dialogs = client.iter_dialogs();
    let mut target_chat = None;

    while let Some(dialog) = dialogs.next().await? {
        match dialog.chat() {
            grammers_client::types::Chat::Channel(ch) => {
                if ch.id() == channel_id {
                    target_chat = Some(dialog.chat().clone());
                    println!("✅ 找到频道: {}", ch.title());
                    break;
                }
            }
            grammers_client::types::Chat::Group(g) => {
                if g.id() == channel_id {
                    target_chat = Some(dialog.chat().clone());
                    println!("✅ 找到群组: {}", g.title());
                    break;
                }
            }
            _ => {}
        }
    }

    let chat = match target_chat {
        Some(c) => c,
        None => {
            println!("❌ 未找到频道/群组 ID: {}", channel_id);
            return Ok(());
        }
    };

    println!("\n📨 开始获取用户历史消息...");
    println!("⏳ 这可能需要一些时间，请稍候...\n");

    // 获取用户的所有消息
    let mut user_messages = Vec::new();
    let mut messages_iter = client.iter_messages(&chat);
    let mut scanned_count = 0;
    let mut found_count = 0;
    let mut username = String::new();
    let mut first_msg_time = String::new();
    let mut last_msg_time = String::new();

    while let Some(message) = messages_iter.next().await? {
        scanned_count += 1;

        // 每扫描100条消息显示进度
        if scanned_count % 100 == 0 {
            println!("📊 已扫描 {} 条消息，找到 {} 条目标用户消息", scanned_count, found_count);
        }

        // 检查是否是目标用户的消息
        if let Some(sender) = message.sender() {
            let sender_id = sender.id();

            if sender_id == target_user_id {
                found_count += 1;

                // 获取用户名（第一次）
                if username.is_empty() {
                    username = match sender {
                        grammers_client::types::Chat::User(user) => {
                            user.first_name().to_string()
                        }
                        grammers_client::types::Chat::Channel(ch) => ch.title().to_string(),
                        grammers_client::types::Chat::Group(g) => g.title().to_string(),
                    };
                }

                let text = message.text().to_string();
                let timestamp = message.date().to_string();
                let has_media = message.media().is_some();
                let is_reply = message.reply_to_message_id().is_some();
                let reply_to_msg_id = message.reply_to_message_id();
                let message_id = message.id();

                // 记录时间范围
                if first_msg_time.is_empty() {
                    first_msg_time = timestamp.clone();
                }
                last_msg_time = timestamp.clone();

                user_messages.push(UserMessage {
                    message_id,
                    timestamp,
                    content: text,
                    has_media,
                    is_reply,
                    reply_to_msg_id,
                });
            }
        }

        // 找到2600条目标用户消息后停止
        if found_count >= 2600 {
            println!("\n✅ 已找到 2600 条目标用户消息，停止扫描");
            break;
        }
    }

    println!("\n✅ 扫描完成！");
    println!("   总扫描: {} 条消息", scanned_count);
    println!("   找到: {} 条目标用户消息\n", found_count);

    if user_messages.is_empty() {
        println!("❌ 未找到用户 {} 的消息", target_user_id);
        return Ok(());
    }

    // 消息按时间倒序排列（最新的在前）
    user_messages.reverse();

    // 统计分析
    let total_count = user_messages.len();
    let with_media = user_messages.iter().filter(|m| m.has_media).count();
    let replies = user_messages.iter().filter(|m| m.is_reply).count();
    
    let total_length: usize = user_messages.iter().map(|m| m.content.len()).sum();
    let avg_message_length = if total_count > 0 {
        total_length as f64 / total_count as f64
    } else {
        0.0
    };

    // 不提取关键词
    let keywords = Vec::new();

    let statistics = MessageStatistics {
        total_count,
        with_media,
        replies,
        avg_message_length,
        keywords_found: keywords,
    };

    // 生成报告
    let report = UserHistoryReport {
        channel_name: "valuescan".to_string(),
        channel_id,
        user_id: target_user_id,
        username: username.clone(),
        total_messages: total_count,
        date_range: format!("{} 至 {}", last_msg_time, first_msg_time),
        messages: user_messages.clone(),
        statistics,
    };

    // 显示摘要
    display_summary(&report);

    // 保存报告
    save_reports(&report)?;

    // 显示最近10条消息
    display_recent_messages(&user_messages, 10);

    Ok(())
}

fn extract_keywords(messages: &[UserMessage]) -> Vec<String> {
    let mut keyword_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for msg in messages {
        let text_lower = msg.content.to_lowercase();

        // 检测币种
        if text_lower.contains("btc") || text_lower.contains("bitcoin") {
            *keyword_counts.entry("BTC".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("eth") || text_lower.contains("ethereum") {
            *keyword_counts.entry("ETH".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("sol") || text_lower.contains("solana") {
            *keyword_counts.entry("SOL".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("usdt") {
            *keyword_counts.entry("USDT".to_string()).or_insert(0) += 1;
        }

        // 检测交易方向
        if text_lower.contains("long") || text_lower.contains("做多") || text_lower.contains("买入") {
            *keyword_counts.entry("做多".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("short") || text_lower.contains("做空") || text_lower.contains("卖出") {
            *keyword_counts.entry("做空".to_string()).or_insert(0) += 1;
        }

        // 检测风控
        if text_lower.contains("止损") || text_lower.contains("stoploss") {
            *keyword_counts.entry("止损".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("止盈") || text_lower.contains("takeprofit") {
            *keyword_counts.entry("止盈".to_string()).or_insert(0) += 1;
        }

        // 检测信号相关
        if text_lower.contains("信号") || text_lower.contains("signal") {
            *keyword_counts.entry("信号".to_string()).or_insert(0) += 1;
        }
        if text_lower.contains("入场") || text_lower.contains("entry") {
            *keyword_counts.entry("入场".to_string()).or_insert(0) += 1;
        }
    }

    // 按出现次数排序
    let mut keywords: Vec<(String, usize)> = keyword_counts.into_iter().collect();
    keywords.sort_by(|a, b| b.1.cmp(&a.1));

    keywords.into_iter().map(|(k, _)| k).collect()
}

fn display_summary(report: &UserHistoryReport) {
    println!("═══════════════════════════════════════════════════");
    println!("📊 用户历史记录摘要");
    println!("═══════════════════════════════════════════════════\n");

    println!("👤 用户信息:");
    println!("  用户名: {}", report.username);
    println!("  用户 ID: {}", report.user_id);
    println!("  频道: {} (ID: {})", report.channel_name, report.channel_id);
    println!();

    println!("📈 统计数据:");
    println!("  总消息数: {} 条", report.statistics.total_count);
    println!("  包含媒体: {} 条", report.statistics.with_media);
    println!("  回复消息: {} 条", report.statistics.replies);
    println!("  平均长度: {:.1} 字符", report.statistics.avg_message_length);
    println!("  时间范围: {}", report.date_range);
    println!();

    if !report.statistics.keywords_found.is_empty() {
        println!("🔑 关键词（按频率）:");
        for (idx, keyword) in report.statistics.keywords_found.iter().enumerate().take(10) {
            println!("  {}. {}", idx + 1, keyword);
        }
        println!();
    }
}

fn display_recent_messages(messages: &[UserMessage], limit: usize) {
    println!("═══════════════════════════════════════════════════");
    println!("📝 最近 {} 条消息", limit.min(messages.len()));
    println!("═══════════════════════════════════════════════════\n");

    for (idx, msg) in messages.iter().take(limit).enumerate() {
        println!("【消息 {}】", idx + 1);
        println!("🕐 时间: {}", msg.timestamp);
        println!("💬 ID: {}", msg.message_id);
        
        if msg.is_reply {
            println!("↩️  回复消息 ID: {:?}", msg.reply_to_msg_id);
        }
        if msg.has_media {
            println!("📎 包含媒体");
        }
        
        println!("\n内容:");
        println!("{}", msg.content);
        println!("\n{}", "─".repeat(50));
        println!();
    }
}

fn save_reports(report: &UserHistoryReport) -> Result<()> {
    // 保存 JSON 格式
    let json_filename = format!("user_{}_history.json", report.user_id);
    let json = serde_json::to_string_pretty(report)?;
    fs::write(&json_filename, json)?;
    println!("✅ JSON 报告已保存: {}", json_filename);

    // 保存文本格式（易读）
    let txt_filename = format!("user_{}_history.txt", report.user_id);
    let mut text = String::new();
    
    text.push_str(&format!("用户历史记录 - {}\n", report.username));
    text.push_str(&format!("用户 ID: {}\n", report.user_id));
    text.push_str(&format!("频道: {} (ID: {})\n", report.channel_name, report.channel_id));
    text.push_str(&format!("总消息数: {}\n", report.total_messages));
    text.push_str(&format!("时间范围: {}\n", report.date_range));
    text.push_str(&format!("\n{}\n\n", "=".repeat(70)));

    for (idx, msg) in report.messages.iter().enumerate() {
        text.push_str(&format!("【消息 {}】\n", idx + 1));
        text.push_str(&format!("时间: {}\n", msg.timestamp));
        text.push_str(&format!("消息 ID: {}\n", msg.message_id));
        
        if msg.is_reply {
            text.push_str(&format!("回复: {:?}\n", msg.reply_to_msg_id));
        }
        if msg.has_media {
            text.push_str("包含媒体: 是\n");
        }
        
        text.push_str(&format!("\n{}\n", msg.content));
        text.push_str(&format!("\n{}\n\n", "-".repeat(70)));
    }

    fs::write(&txt_filename, text)?;
    println!("✅ 文本报告已保存: {}", txt_filename);

    Ok(())
}
