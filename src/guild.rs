use crate::{async_trait, GuildMap, PContext, TypeMap};
use serenity::model::id::GuildId;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SQGuild {
    pub message_count: u64,
}

impl SQGuild {
    pub fn new() -> Self {
        SQGuild { message_count: 0 }
    }
}

#[async_trait]
pub trait IntoSQGuild<'a> {
    async fn get_sq_guild(&self) -> Arc<RwLock<SQGuild>>;
}

#[async_trait]
impl<'a> IntoSQGuild<'a> for PContext<'a> {
    async fn get_sq_guild(&self) -> Arc<RwLock<SQGuild>> {
        let data = &self.discord().data;
        let guild_id = self.guild_id().expect("Guild ID not present");
        get_or_init_sqguild(data, &guild_id).await
    }
}

pub async fn get_or_init_sqguild(
    data: &Arc<RwLock<TypeMap>>,
    guild_id: &GuildId,
) -> Arc<RwLock<SQGuild>> {
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
            let arc = guild_map
                .entry(*guild_id)
                .or_insert(Arc::new(RwLock::new(sq_guild)));
            Arc::clone(arc)
        }
    };
}
