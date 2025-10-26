use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{types::Chat, Client, Config, InitParams};
use grammers_session::Session;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // 读取配置
    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE")?;

    println!("🔍 获取账号 {} 的频道列表...\n", phone);

    // 连接到 Telegram，使用官方客户端参数
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
        println!("❌ 账号未登录，请先运行 signal_trader 进行登录");
        return Ok(());
    }

    println!("✅ 账号已登录，正在获取频道列表...\n");

    // 获取所有对话
    let mut dialogs = client.iter_dialogs();
    let mut channel_count = 0;
    let mut total_count = 0;

    println!("📋 频道列表:");
    println!("════════════════════════════════════════");

    while let Some(dialog) = dialogs.next().await? {
        total_count += 1;

        // 检查是否是频道
        if let Chat::Channel(channel) = dialog.chat() {
            channel_count += 1;
            let title = dialog.chat().name();
            let id = channel.id();
            // 特别标记目标频道
            let marker = if id == 2291145819 {
                " 🎯 [目标频道]"
            } else {
                ""
            };

            println!("{:3}. {} (ID: {})", channel_count, title, id);
            println!("     👥 标记: {}", marker.trim());
            println!();
        }
    }

    println!("════════════════════════════════════════");
    println!("📊 统计信息:");
    println!("   频道数量: {} 个", channel_count);
    println!("   对话总数: {} 个", total_count);
    println!("   私聊/群组: {} 个", total_count - channel_count);

    Ok(())
}
