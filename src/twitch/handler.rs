use std::fmt::Display;

use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use crate::twitch::bot::{MuniBotTwitchIRCClient, MuniBotTwitchIRCError};

#[async_trait]
pub trait TwitchMessageHandler: Send {
    async fn send_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        channel: &str,
        message: &str,
    ) -> Result<(), TwitchHandlerError> {
        client
            .say(channel.to_string(), message.to_string())
            .await
            .map_err(TwitchHandlerError::SendMessage)
    }

    /// Handle a new message from chat. Returns `true` if something was done to handle the message,
    /// or `false` if the message was ignored (or if the message is allowed to also be handled by
    /// other handlers).
    async fn handle_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, TwitchHandlerError>;
}

#[derive(Debug)]
pub enum TwitchHandlerError {
    SendMessage(MuniBotTwitchIRCError),
    TwitchIRCError(MuniBotTwitchIRCError),
}

impl From<MuniBotTwitchIRCError> for TwitchHandlerError {
    fn from(e: MuniBotTwitchIRCError) -> Self {
        Self::TwitchIRCError(e)
    }
}

impl Display for TwitchHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwitchHandlerError::SendMessage(e) => write!(f, "message send failure! {e}"),
            TwitchHandlerError::TwitchIRCError(e) => write!(f, "irc error :< {e}"),
        }
    }
}

impl std::error::Error for TwitchHandlerError {}
