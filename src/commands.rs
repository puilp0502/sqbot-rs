use anyhow::Error;
use serenity::model::prelude::*;

use crate::guild::IntoSQGuild;

pub type PContext<'a> = poise::Context<'a, (), Error>;

#[poise::command(slash_command)]
pub async fn age(
    ctx: PContext<'_>,
    #[description = "Selected user"] user: Option<User>,
) -> anyhow::Result<()> {
    let sq_guild = ctx.get_sq_guild().await;
    println!("Guild: {:?}", sq_guild.read().await);
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {:?}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: PContext<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
