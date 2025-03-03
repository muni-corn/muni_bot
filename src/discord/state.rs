use std::sync::Arc;

use surrealdb::{engine::remote::ws, opt::auth::Database, Surreal};
use tokio::sync::Mutex;

use super::handler::DiscordEventHandler;
use crate::{
    config::{Config, DiscordConfig},
    handlers::DiscordMessageHandlerCollection,
    MuniBotError,
};

pub struct DiscordState {
    pub config: DiscordConfig,
    handlers: DiscordMessageHandlerCollection,
    pub db: Surreal<ws::Client>,
}
impl DiscordState {
    pub async fn new(
        handlers: DiscordMessageHandlerCollection,
        config: &Config,
    ) -> Result<Self, MuniBotError> {
        let database_url = config.db.url.clone();
        let db = Surreal::new::<ws::Ws>(&database_url).await?;
        db.signin(Database {
            namespace: "muni_bot",
            database: "muni_bot",
            username: &config.db.user,
            password: &std::env::var("DATABASE_PASS").expect("expected DATABASE_PASS to be set"),
        })
        .await?;

        Ok(Self {
            handlers,
            db,
            config: config.discord.clone(),
        })
    }

    pub fn handlers(&self) -> &[Arc<Mutex<dyn DiscordEventHandler>>] {
        &self.handlers
    }
}
