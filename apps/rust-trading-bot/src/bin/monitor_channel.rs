use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{Client, Config, Update};
use grammers_session::Session;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("📡 Telegram 频道消息监听器\n");

    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE")?;
    let target_channel_id: i64 = env::var("TARGET_CHANNEL_ID")
        .unwrap_or_else(|_| "2291145819".to_string())
        .parse()?;

    println!("🔑 API ID: {}", api_id);
    println!("📱 手机号: {}", phone);
    println!("🎯 监听频道 ID: {}\n", target_channel_id);

    println!("🔄 连接到 Telegram...");

    let client = Client::connect(Config {
        session: Session::load_file_or_create("session.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    println!("✅ 连接成功");

    if !client.is_authorized().await? {
        println!("⚠️  需要登录");
        println!("📨 发送验证码到 {}...", phone);

        let token = client.request_login_code(&phone).await?;
        println!("✅ 验证码已发送");

        println!("\n🔢 请输入收到的验证码:");
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        let code = code.trim();

        client.sign_in(&token, code).await?;
        println!("✅ 登录成功!");
        client.session().save_to_file("session.session")?;
    } else {
        println!("✅ 已登录");
    }

    println!("\n🔍 开始监听频道消息...");
    println!("════════════════════════════════════════\n");

    loop {
        match client.next_update().await? {
            Update::NewMessage(message) if !message.outgoing() => {
                let chat = message.chat();

                // 检查是否是目标频道
                match chat {
                    grammers_client::types::Chat::Channel(channel) => {
                        if channel.id() == target_channel_id {
                            println!(
                                "📨 [{}] 新消息",
                                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
                            );
                            println!("   频道: {}", channel.title());

                            let text = message.text();
                            println!("   内容:\n{}", text);

                            println!("   ────────────────────────────────────");
                            println!();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
