use std::sync::Arc;

use poise::serenity_prelude::*;
use surrealdb::{engine::remote::ws, opt::auth::Database, Surreal};
use tokio::sync::Mutex;

use super::handler::DiscordEventHandler;
use crate::{
    config::{Config, DiscordConfig},
    handlers::{logging::LoggingHandler, DiscordMessageHandlerCollection},
    MuniBotError,
};

pub struct DiscordState {
    pub config: DiscordConfig,
    pub db: Surreal<ws::Client>,

    handlers: DiscordMessageHandlerCollection,
    logging: Arc<Mutex<LoggingHandler>>,
}
impl DiscordState {
    /// creates a new `DiscordState` struct. a `LoggingHandler` is added for
    /// you.
    pub async fn new(
        mut handlers: DiscordMessageHandlerCollection,
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

        // add the logging handler to the list of handlers
        let logging = Arc::new(Mutex::new(LoggingHandler));
        handlers.push(logging.clone());

        Ok(Self {
            handlers,
            db,
            config: config.discord.clone(),
            logging,
        })
    }

    pub fn handlers(&self) -> &[Arc<Mutex<dyn DiscordEventHandler>>] {
        &self.handlers
    }

    pub fn logging(&self) -> Arc<Mutex<LoggingHandler>> {
        self.logging.clone()
    }
}
