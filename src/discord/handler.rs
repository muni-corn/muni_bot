use std::{
    error::Error,
    fmt::{self, Display},
};

use async_trait::async_trait;
use poise::serenity_prelude::{Context, Message};

#[async_trait]
pub trait DiscordMessageHandler: Sync + Send {
    fn name(&self) -> &'static str;
    async fn handle_discord_message(
        &mut self,
        context: &Context,
        msg: &Message,
    ) -> Result<bool, DiscordMessageHandlerError>;
}

#[derive(Debug)]
pub struct DiscordMessageHandlerError {
    pub message: String,
    pub handler_name: String,
}
impl Display for DiscordMessageHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} discord message handler encountered an error: {}",
            self.handler_name, self.message
        )
    }
}
impl Error for DiscordMessageHandlerError {}
