use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{types::Message, Client, Config, InitParams};
use grammers_session::Session;
use std::collections::HashMap;
use std::env;

/// 频道配置
#[derive(Debug, Clone)]
struct ChannelConfig {
    id: i64,
    name: String,
    channel_type: ChannelType,
}

#[derive(Debug, Clone, PartialEq)]
enum ChannelType {
    Primary,   // 主频道
    Auxiliary, // 辅助频道
}

/// 用户消息统计
#[derive(Debug, Default)]
struct UserStats {
    username: String,
    user_id: i64,
    message_count: usize,
    messages: Vec<MessageInfo>,
}

#[derive(Debug, Clone)]
struct MessageInfo {
    text: String,
    date: String,
    channel: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("═══════════════════════════════════════════════════");
    println!("📊 Telegram 多频道分析与用户消息总结工具");
    println!("═══════════════════════════════════════════════════\n");

    // 配置频道
    let channels = vec![
        ChannelConfig {
            id: 2254462672,
            name: "valuescan".to_string(),
            channel_type: ChannelType::Primary,
        },
        ChannelConfig {
            id: 2291145819,
            name: "CM AI SIGNAL".to_string(),
            channel_type: ChannelType::Auxiliary,
        },
    ];

    println!("📡 频道配置:");
    for channel in &channels {
        let type_label = match channel.channel_type {
            ChannelType::Primary => "🎯 主频道",
            ChannelType::Auxiliary => "🔧 辅助频道",
        };
        println!("  {} {} (ID: {})", type_label, channel.name, channel.id);
    }
    println!();

    // 连接到 Telegram
    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE")?;

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
        println!("❌ 账号未登录，请先运行 list_channels 进行登录");
        return Ok(());
    }

    println!("✅ 连接成功\n");

    // 分析每个频道
    for channel_config in &channels {
        println!("════════════════════════════════════════════════════");
        println!(
            "📊 分析频道: {} ({})",
            channel_config.name,
            match channel_config.channel_type {
                ChannelType::Primary => "主频道",
                ChannelType::Auxiliary => "辅助频道",
            }
        );
        println!("════════════════════════════════════════════════════\n");

        analyze_channel(&client, channel_config).await?;
    }

    Ok(())
}

async fn analyze_channel(client: &Client, config: &ChannelConfig) -> Result<()> {
    // 获取频道实体
    let channel = match client.resolve_username(&config.id.to_string()).await {
        Ok(chat) => chat,
        Err(_) => {
            // 如果用户名解析失败，尝试通过对话列表查找
            println!("🔍 通过 ID 查找频道...");
            let mut found = None;
            let mut dialogs = client.iter_dialogs();

            while let Some(dialog) = dialogs.next().await? {
                if let grammers_client::types::Chat::Channel(ch) = dialog.chat() {
                    if ch.id() == config.id {
                        found = Some(dialog.chat().clone());
                        break;
                    }
                }
            }

            match found {
                Some(chat) => chat,
                None => {
                    println!("❌ 无法找到频道 {}", config.name);
                    return Ok(());
                }
            }
        }
    };

    println!("📨 获取最近消息（限制100条）...\n");

    // 获取历史消息
    let mut messages_iter = client.iter_messages(&channel);
    let mut user_messages: HashMap<i64, UserStats> = HashMap::new();
    let mut total_messages = 0;
    let limit = 100;

    while let Some(message) = messages_iter.next().await? {
        total_messages += 1;
        if total_messages > limit {
            break;
        }

        // 只处理用户消息（非系统消息）
        if let Some(sender) = message.sender() {
            let user_id = sender.id();
            let username = match sender {
                grammers_client::types::Chat::User(user) => user
                    .username()
                    .unwrap_or(&user.first_name().unwrap_or("Unknown"))
                    .to_string(),
                grammers_client::types::Chat::Channel(ch) => ch.title().to_string(),
                grammers_client::types::Chat::Group(g) => g.title().to_string(),
            };

            let text = message.text().to_string();
            let date = message.date().to_string();

            let stats = user_messages.entry(user_id).or_insert_with(|| UserStats {
                username: username.clone(),
                user_id,
                message_count: 0,
                messages: Vec::new(),
            });

            stats.message_count += 1;
            stats.messages.push(MessageInfo {
                text,
                date,
                channel: config.name.clone(),
            });
        }
    }

    // 显示统计结果
    println!("📊 频道统计:");
    println!("  总消息数: {} 条", total_messages);
    println!("  活跃用户: {} 人\n", user_messages.len());

    // 按消息数量排序用户
    let mut sorted_users: Vec<_> = user_messages.values().collect();
    sorted_users.sort_by(|a, b| b.message_count.cmp(&a.message_count));

    println!("👥 用户活跃度排名:");
    println!("─────────────────────────────────────────────────");

    for (idx, user) in sorted_users.iter().enumerate().take(10) {
        println!(
            "{}. {} (@{}) - {} 条消息",
            idx + 1,
            user.username,
            user.user_id,
            user.message_count
        );
    }
    println!();

    // 显示每个用户的消息总结
    println!("📝 用户消息详细总结:");
    println!("═════════════════════════════════════════════════\n");

    for (idx, user) in sorted_users.iter().enumerate().take(5) {
        println!(
            "【用户 {}】{} (ID: {})",
            idx + 1,
            user.username,
            user.user_id
        );
        println!("消息总数: {} 条", user.message_count);
        println!("\n最近 5 条消息:");
        println!("─────────────────────────────────────────────────");

        for (msg_idx, msg) in user.messages.iter().take(5).enumerate() {
            println!("\n[{}] {}", msg_idx + 1, msg.date);
            let preview = if msg.text.len() > 100 {
                format!("{}...", &msg.text[..100])
            } else {
                msg.text.clone()
            };
            println!("{}", preview);
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    Ok(())
}
