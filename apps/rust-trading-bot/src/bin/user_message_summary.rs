use anyhow::Result;
use chrono::Utc;
use dotenv::dotenv;
use grammers_client::{Client, Config, InitParams};
use grammers_session::Session;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

/// 频道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChannelConfig {
    id: i64,
    name: String,
    role: String, // "primary" 或 "auxiliary"
}

/// 用户消息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSummary {
    user_id: i64,
    username: String,
    display_name: String,
    total_messages: usize,
    channels: HashMap<String, ChannelActivity>,
    message_timeline: Vec<MessageRecord>,
    keywords: Vec<String>,
    activity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChannelActivity {
    channel_name: String,
    message_count: usize,
    last_active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageRecord {
    timestamp: String,
    channel: String,
    content: String,
    is_reply: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisReport {
    generated_at: String,
    channels: Vec<ChannelConfig>,
    user_summaries: Vec<UserSummary>,
    statistics: Statistics,
}

#[derive(Debug, Serialize, Deserialize)]
struct Statistics {
    total_messages: usize,
    total_users: usize,
    avg_messages_per_user: f64,
    most_active_user: String,
    most_active_channel: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("═══════════════════════════════════════════════════");
    println!("📊 群组/频道用户消息分析与总结工具");
    println!("═══════════════════════════════════════════════════\n");

    // 配置要分析的频道
    let channels = vec![
        ChannelConfig {
            id: 2254462672,
            name: "valuescan".to_string(),
            role: "primary".to_string(),
        },
        ChannelConfig {
            id: 2291145819,
            name: "CM AI SIGNAL".to_string(),
            role: "auxiliary".to_string(),
        },
    ];

    println!("🎯 目标频道:");
    for ch in &channels {
        let emoji = if ch.role == "primary" { "⭐" } else { "🔧" };
        println!("  {} {} - {} (ID: {})", emoji, ch.role, ch.name, ch.id);
    }
    println!();

    // 连接 Telegram
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

    // 分析所有频道
    let mut all_user_data: HashMap<i64, UserSummary> = HashMap::new();
    let mut total_messages = 0;

    for channel_config in &channels {
        println!(
            "📡 分析频道: {} ({})...",
            channel_config.name, channel_config.role
        );

        let user_data = analyze_channel_messages(&client, channel_config).await?;

        // 合并用户数据
        for (user_id, summary) in user_data {
            total_messages += summary.total_messages;

            let entry = all_user_data.entry(user_id).or_insert_with(|| UserSummary {
                user_id,
                username: summary.username.clone(),
                display_name: summary.display_name.clone(),
                total_messages: 0,
                channels: HashMap::new(),
                message_timeline: Vec::new(),
                keywords: Vec::new(),
                activity_score: 0.0,
            });

            entry.total_messages += summary.total_messages;
            entry.channels.extend(summary.channels);
            entry.message_timeline.extend(summary.message_timeline);
            entry.keywords.extend(summary.keywords);
        }
    }

    println!("\n✅ 数据收集完成\n");

    // 计算活跃度分数
    for user_summary in all_user_data.values_mut() {
        user_summary.activity_score = calculate_activity_score(user_summary);

        // 去重关键词
        user_summary.keywords.sort();
        user_summary.keywords.dedup();

        // 按时间排序消息
        user_summary
            .message_timeline
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    // 生成报告
    let report = generate_report(channels, all_user_data, total_messages);

    // 显示摘要
    display_summary(&report);

    // 保存报告
    save_report(&report)?;

    Ok(())
}

async fn analyze_channel_messages(
    client: &Client,
    config: &ChannelConfig,
) -> Result<HashMap<i64, UserSummary>> {
    let mut user_data: HashMap<i64, UserSummary> = HashMap::new();

    // 查找频道
    let mut dialogs = client.iter_dialogs();
    let mut target_chat = None;

    while let Some(dialog) = dialogs.next().await? {
        if let grammers_client::types::Chat::Channel(ch) = dialog.chat() {
            if ch.id() == config.id {
                target_chat = Some(dialog.chat().clone());
                break;
            }
        }
        if let grammers_client::types::Chat::Group(g) = dialog.chat() {
            if g.id() == config.id {
                target_chat = Some(dialog.chat().clone());
                break;
            }
        }
    }

    let chat = match target_chat {
        Some(c) => c,
        None => {
            println!("  ⚠️  未找到频道 {}", config.name);
            return Ok(user_data);
        }
    };

    // 获取消息
    let mut messages_iter = client.iter_messages(&chat);
    let mut count = 0;
    let limit = 200; // 分析最近 200 条消息

    while let Some(message) = messages_iter.next().await? {
        count += 1;
        if count > limit {
            break;
        }

        if let Some(sender) = message.sender() {
            let user_id = sender.id();

            let (username, display_name) = match sender {
                grammers_client::types::Chat::User(user) => {
                    let uname = user.username().unwrap_or("").to_string();
                    let mut display_name = user.first_name().trim().to_string();
                    if let Some(last_name) = user.last_name() {
                        if !last_name.trim().is_empty() {
                            display_name = format!("{} {}", display_name, last_name.trim());
                        }
                    }
                    if display_name.trim().is_empty() {
                        display_name = user.username().unwrap_or("Unknown").to_string();
                    }
                    (uname, display_name)
                }
                grammers_client::types::Chat::Channel(ch) => {
                    let title = ch.title().to_string();
                    (title.clone(), title)
                }
                grammers_client::types::Chat::Group(g) => {
                    let title = g.title().to_string();
                    (title.clone(), title)
                }
            };

            let text = message.text().to_string();
            let timestamp = message.date().to_string();
            let is_reply = message.reply_to_message_id().is_some();

            // 提取关键词
            let keywords = extract_keywords(&text);

            let summary = user_data.entry(user_id).or_insert_with(|| UserSummary {
                user_id,
                username: username.clone(),
                display_name: display_name.clone(),
                total_messages: 0,
                channels: HashMap::new(),
                message_timeline: Vec::new(),
                keywords: Vec::new(),
                activity_score: 0.0,
            });

            summary.total_messages += 1;

            let channel_activity =
                summary
                    .channels
                    .entry(config.name.clone())
                    .or_insert_with(|| ChannelActivity {
                        channel_name: config.name.clone(),
                        message_count: 0,
                        last_active: timestamp.clone(),
                    });

            channel_activity.message_count += 1;
            channel_activity.last_active = timestamp.clone();

            summary.message_timeline.push(MessageRecord {
                timestamp,
                channel: config.name.clone(),
                content: text,
                is_reply,
            });

            summary.keywords.extend(keywords);
        }
    }

    println!("  ✅ 收集 {} 条消息", count);
    Ok(user_data)
}

fn extract_keywords(text: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // 简单的关键词提取
    let words: Vec<&str> = text.split_whitespace().collect();

    for word in words {
        let word_lower = word.to_lowercase();

        // 检测加密货币相关关键词
        if word_lower.contains("btc") || word_lower.contains("bitcoin") {
            keywords.push("BTC".to_string());
        }
        if word_lower.contains("eth") || word_lower.contains("ethereum") {
            keywords.push("ETH".to_string());
        }
        if word_lower.contains("usdt") {
            keywords.push("USDT".to_string());
        }

        // 检测交易信号
        if word_lower.contains("buy") || word_lower.contains("long") || word.contains("做多") {
            keywords.push("做多信号".to_string());
        }
        if word_lower.contains("sell") || word_lower.contains("short") || word.contains("做空") {
            keywords.push("做空信号".to_string());
        }
        if word.contains("止损") {
            keywords.push("止损".to_string());
        }
        if word.contains("止盈") {
            keywords.push("止盈".to_string());
        }
    }

    keywords
}

fn calculate_activity_score(summary: &UserSummary) -> f64 {
    let message_weight = 1.0;
    let channel_diversity_weight = 5.0;
    let reply_weight = 2.0;

    let message_score = summary.total_messages as f64 * message_weight;
    let channel_score = summary.channels.len() as f64 * channel_diversity_weight;
    let reply_count = summary
        .message_timeline
        .iter()
        .filter(|m| m.is_reply)
        .count();
    let reply_score = reply_count as f64 * reply_weight;

    message_score + channel_score + reply_score
}

fn generate_report(
    channels: Vec<ChannelConfig>,
    user_data: HashMap<i64, UserSummary>,
    total_messages: usize,
) -> AnalysisReport {
    let mut summaries: Vec<UserSummary> = user_data.into_values().collect();
    summaries.sort_by(|a, b| b.activity_score.partial_cmp(&a.activity_score).unwrap());

    let total_users = summaries.len();
    let avg_messages = if total_users > 0 {
        total_messages as f64 / total_users as f64
    } else {
        0.0
    };

    let most_active_user = summaries
        .first()
        .map(|s| s.display_name.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let most_active_channel = channels
        .iter()
        .find(|c| c.role == "primary")
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "N/A".to_string());

    AnalysisReport {
        generated_at: Utc::now().to_rfc3339(),
        channels,
        user_summaries: summaries,
        statistics: Statistics {
            total_messages,
            total_users,
            avg_messages_per_user: avg_messages,
            most_active_user,
            most_active_channel,
        },
    }
}

fn display_summary(report: &AnalysisReport) {
    println!("═══════════════════════════════════════════════════");
    println!("📊 分析报告摘要");
    println!("═══════════════════════════════════════════════════\n");

    println!("📈 总体统计:");
    println!("  消息总数: {}", report.statistics.total_messages);
    println!("  用户总数: {}", report.statistics.total_users);
    println!(
        "  平均消息数: {:.1}",
        report.statistics.avg_messages_per_user
    );
    println!("  最活跃用户: {}", report.statistics.most_active_user);
    println!();

    println!("👥 Top 10 活跃用户:");
    println!("─────────────────────────────────────────────────");

    for (idx, user) in report.user_summaries.iter().take(10).enumerate() {
        println!(
            "{}. {} (@{}) - {} 条消息 | 活跃度: {:.1}",
            idx + 1,
            user.display_name,
            user.username,
            user.total_messages,
            user.activity_score
        );

        if !user.keywords.is_empty() {
            println!("   关键词: {}", user.keywords.join(", "));
        }

        println!(
            "   活跃频道: {}",
            user.channels.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        println!();
    }

    println!("\n💡 详细报告已保存到 channel_analysis_report.json");
}

fn save_report(report: &AnalysisReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    fs::write("channel_analysis_report.json", json)?;
    println!("✅ 报告已保存");
    Ok(())
}
