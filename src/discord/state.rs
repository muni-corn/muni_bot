use std::sync::Arc;

use poise::serenity_prelude::{Cache, Http, Result};
use surrealdb::{engine::remote::ws, Surreal};
use tokio::sync::Mutex;

use super::handler::DiscordEventHandler;
use crate::{
    config::{Config, DiscordConfig},
    handlers::{logging::LoggingHandler, DiscordMessageHandlerCollection},
    MuniBotError,
};

#[derive(Clone, Debug)]
pub struct GlobalAccess {
    db: Arc<Surreal<ws::Client>>,
    http: Arc<Http>,
    cache: Arc<Cache>,
}

impl GlobalAccess {
    pub fn new(http: Arc<Http>, cache: Arc<Cache>, db: Arc<Surreal<ws::Client>>) -> Self {
        Self { http, cache, db }
    }

    pub fn db(&self) -> &Surreal<ws::Client> {
        &self.db
    }

    pub fn http(&self) -> &Http {
        &self.http
    }

    pub fn cache(&self) -> &Arc<Cache> {
        &self.cache
    }

    pub fn as_cache_http(&self) -> (&Arc<Cache>, &Http) {
        (&self.cache, &*self.http)
    }
}

pub struct DiscordState {
    pub config: DiscordConfig,
    handlers: DiscordMessageHandlerCollection,
    access: GlobalAccess,

    logging: Arc<Mutex<LoggingHandler>>,
}
impl DiscordState {
    /// creates a new `DiscordState` struct. a `LoggingHandler` is added for
    /// you.
    pub async fn new(
        mut handlers: DiscordMessageHandlerCollection,
        config: &Config,
        db: Arc<Surreal<ws::Client>>,
        http: Arc<Http>,
        cache: Arc<Cache>,
    ) -> Result<Self, MuniBotError> {
        let global_access = GlobalAccess { db, http, cache };

        // add the logging handler to the list of handlers
        let logging = Arc::new(Mutex::new(LoggingHandler));

        handlers.push(logging.clone());

        Ok(Self {
            handlers,
            config: config.discord.clone(),
            access: global_access,
            logging,
        })
    }

    pub fn handlers(&self) -> &[Arc<Mutex<dyn DiscordEventHandler>>] {
        &self.handlers
    }

    pub fn access(&self) -> &GlobalAccess {
        &self.access
    }

    pub fn logging(&self) -> &Arc<Mutex<LoggingHandler>> {
        &self.logging
    }
}
