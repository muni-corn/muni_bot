pub mod greeting;
pub mod lurk;

use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use crate::bot::MuniBotTwitchIRCClient;

#[async_trait]
pub trait MessageHandler: Send {
    /// Handle a new message from chat. Returns `true` if something was done to handle the message,
    /// or `false` if the message was ignored (or if the message is allowed to also be handled by
    /// other handlers).
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> bool;
}
