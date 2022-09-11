use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use anyhow::Error;
use poise::PrefixFrameworkOptions;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct GuildMap;

#[derive(Debug)]
struct SQGuild {
    message_count: u64,
}

impl SQGuild {
    fn new() -> Self {
        SQGuild {
            message_count: 0
        }
    }
}

impl TypeMapKey for GuildMap {
    type Value = HashMap<GuildId, Arc<RwLock<SQGuild>>>;
}

type PContext<'a> = poise::Context<'a,  (), Error>;

async fn get_or_init_sqguild(data: &Arc<RwLock<TypeMap>>, guild_id: &GuildId) -> Arc<RwLock<SQGuild>> {
    let ro_typemap = data.read().await;
    let guild_map = ro_typemap.get::<GuildMap>().unwrap();
    return match guild_map.get(guild_id) {
        Some(guild) => Arc::clone(guild),
        None => {
            drop(ro_typemap);
            let mut rw_typemap = data.write().await;
            let guild_map = rw_typemap.get_mut::<GuildMap>().unwrap();
            // We're doing optimistic locking, so we need to check again
            // if the SQGuild has been initialized by other task
            let sq_guild = SQGuild::new();

            // We could lazily init SQGuild by using `.or_insert_with`,
            // but creating one is cheap enough, so we just create SQGuild unconditionally.
            let arc = guild_map.entry(*guild_id)
                .or_insert(Arc::new(RwLock::new(sq_guild)));
            Arc::clone(arc)
        },
    };
}

#[poise::command(slash_command)]
async fn age(
    ctx: PContext<'_>,
    #[description = "Selected user"] user: Option<User>,
) -> anyhow::Result<()> {
    let guild_id = ctx.guild_id().unwrap();
    let sq_guild = get_or_init_sqguild(&ctx.discord().data, &guild_id).await;
    println!("Guild: {:?}", sq_guild.read().await);
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

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.cache.current_user_id() {
            return
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

    framework.run().await.unwrap();
}
