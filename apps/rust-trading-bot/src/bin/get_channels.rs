use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("📱 Telegram 频道列表获取工具\n");

    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;
    let phone = env::var("TELEGRAM_PHONE").unwrap_or_else(|_| "+18489994567".to_string());

    println!("🔑 API ID: {}", api_id);
    println!("🔑 API Hash: {}...", &api_hash[..8]);
    println!("📱 手机号: {}\n", phone);

    println!("🔄 连接到 Telegram...");

    let client = Client::connect(Config {
        session: Session::load_file_or_create("session.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    println!("✅ 连接成功\n");

    if !client.is_authorized().await? {
        println!("⚠️  需要登录到 {}", phone);

        println!("📨 发送验证码...");
        let token = client.request_login_code(&phone).await?;
        println!("✅ 验证码已发送");

        println!("\n🔢 请输入收到的验证码:");
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        let code = code.trim();

        match client.sign_in(&token, code).await {
            Ok(_) => {
                println!("✅ 登录成功!");
                client.session().save_to_file("session.session")?;
            }
            Err(SignInError::PasswordRequired(password_token)) => {
                println!("🔒 需要两步验证密码:");
                let mut password = String::new();
                std::io::stdin().read_line(&mut password)?;
                let password = password.trim();

                client.check_password(password_token, password).await?;
                println!("✅ 登录成功!");
                client.session().save_to_file("session.session")?;
            }
            Err(e) => return Err(e.into()),
        }
    } else {
        println!("✅ 已登录\n");
    }

    println!("📂 获取频道列表...\n");

    let mut dialogs = client.iter_dialogs();
    let mut channel_count = 0;

    while let Some(dialog) = dialogs.next().await? {
        let chat = dialog.chat();

        // 只显示频道和群组
        match chat {
            grammers_client::types::Chat::Channel(channel) => {
                channel_count += 1;
                println!("{}. 📢 {}", channel_count, channel.title());
                println!("   ID: {}", channel.id());

                if let Some(username) = channel.username() {
                    println!("   Username: @{}", username);
                }

                println!();
            }
            grammers_client::types::Chat::Group(group) => {
                channel_count += 1;
                println!("{}. 👥 {}", channel_count, group.title());
                println!("   ID: {}", group.id());
                println!();
            }
            _ => {}
        }
    }

    println!("════════════════════════════════════════");
    println!("✅ 找到 {} 个频道", channel_count);

    Ok(())
}
