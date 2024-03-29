use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, Message};
use thiserror::Error;

use super::DiscordFrameworkContext;

#[async_trait]
pub trait DiscordMessageHandler: Sync + Send {
    fn name(&self) -> &'static str;
    async fn handle_discord_message(
        &mut self,
        context: &serenity::Context,
        framework: DiscordFrameworkContext<'_>,
        msg: &Message,
    ) -> Result<bool, DiscordMessageHandlerError>;
}

#[derive(Error, Debug)]
#[error("error in discord handler {handler_name}: {message}")]
pub struct DiscordMessageHandlerError {
    pub handler_name: &'static str,
    pub message: String,
}
