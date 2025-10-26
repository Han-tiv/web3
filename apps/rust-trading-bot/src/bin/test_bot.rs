use dotenv::dotenv;
use std::env;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("🤖 测试 Telegram Bot 连接...\n");

    let token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN 未设置");

    println!("📱 Token: {}...", &token[..20]);

    let bot = Bot::new(token);

    match bot.get_me().await {
        Ok(me) => {
            println!("✅ Bot 连接成功!\n");
            println!("   Bot 用户名: @{}", me.username.as_deref().unwrap_or("无"));
            println!("   Bot 名称: {}", me.first_name);
            println!("   Bot ID: {}", me.id);
            println!("\n💬 现在可以在 Telegram 中向 Bot 发送消息测试");
        }
        Err(e) => {
            println!("❌ Bot 连接失败: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n🔄 启动消息监听...\n");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            println!("📨 收到消息: {}", text);
            bot.send_message(msg.chat.id, format!("你说: {}", text))
                .await?;
        }
        Ok(())
    })
    .await;
}
