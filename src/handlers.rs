pub mod greeting;
pub mod lurk;
pub mod raid_msg;

use std::fmt::Display;

use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use crate::bot::{MuniBotTwitchIRCClient, MuniBotTwitchIRCError};

#[async_trait]
pub trait MessageHandler: Send {
    async fn send_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        channel: &str,
        message: &str,
    ) -> Result<(), HandlerError> {
        client
            .say(channel.to_string(), message.to_string())
            .await
            .map_err(|e| HandlerError::SendMessage(e))
    }

    /// Handle a new message from chat. Returns `true` if something was done to handle the message,
    /// or `false` if the message was ignored (or if the message is allowed to also be handled by
    /// other handlers).
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, HandlerError>;
}

#[derive(Debug)]
pub enum HandlerError {
    SendMessage(MuniBotTwitchIRCError),
    TwitchIRCError(MuniBotTwitchIRCError),
}

impl From<MuniBotTwitchIRCError> for HandlerError {
    fn from(e: MuniBotTwitchIRCError) -> Self {
        Self::TwitchIRCError(e)
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::SendMessage(e) => write!(f, "message send failure! {e}"),
            HandlerError::TwitchIRCError(e) => write!(f, "irc error :< {e}"),
        }
    }
}

impl std::error::Error for HandlerError {}
