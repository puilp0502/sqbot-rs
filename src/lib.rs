use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Error;
use poise::{FrameworkBuilder, PrefixFrameworkOptions};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::prelude::*;
use serenity::prelude::*;

use commands::*;
use guild::get_or_init_sqguild;
use guild::SQGuild;

mod commands;
mod guild;

struct GuildMap;

impl TypeMapKey for GuildMap {
    type Value = HashMap<GuildId, Arc<RwLock<SQGuild>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user_id() {
            return;
        }
        let arc = get_or_init_sqguild(&ctx.data, &msg.guild_id.unwrap()).await;
        let mut guild = arc.write().await;
        guild.message_count += 1;
        println!("Author: {:?}", msg.author.id);
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx, "Pong!").await {
                eprintln!("Error sending message: {:?}", why)
            }
        }
    }
}

pub fn create_framework(token: &str) -> FrameworkBuilder<(), Error> {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), register()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("&".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .client_settings(|builder| {
            builder
                .event_handler(Handler {})
                .type_map_insert::<GuildMap>(HashMap::new())
        })
        .user_data_setup(|_ctx, _ready, _framework| Box::pin(async { Ok(()) }));

    framework
}
