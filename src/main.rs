use std::env;

use serenity_test::create_framework;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_BOT_TOKEN")
        .expect("Bot token not found! Set DISCORD_BOT_TOKEN environment variable.");
    let framework = create_framework(&token);

    framework.run().await.unwrap();
}
