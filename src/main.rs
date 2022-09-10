use std::env;

use anyhow::Error;
use poise::PrefixFrameworkOptions;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

struct Data {}
type PContext<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn age(
    ctx: PContext<'_>,
    #[description = "Selected user"] user: Option<poise::serenity_prelude::User>,
) -> anyhow::Result<()> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {:?}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: PContext<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx, "Pong!").await {
                eprintln!("Error sending message: {:?}", why)
            }
        }
        ()
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_BOT_TOKEN")
        .expect("Bot token not found! Set DISCORD_BOT_TOKEN environment variable.");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), register()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("~".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .client_settings(|builder| builder.event_handler(Handler {}))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}
