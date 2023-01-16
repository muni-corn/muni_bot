use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use super::{HandlerError, MessageHandler};
use crate::bot::MuniBotTwitchIRCClient;

pub struct LurkHandler;

#[async_trait]
impl MessageHandler for LurkHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, HandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!lurk") {
                self.send_message(
                    client,
                    &m.channel_login,
                    &format!("{} cast an invisibility spell!", m.sender.name),
                )
                .await?;
                true
            } else if m.message_text.trim().starts_with("!unlurk") {
                self.send_message(
                    client,
                    &m.channel_login,
                    &format!(
                        "{}'s invisibility spell wore off. we can see you!",
                        m.sender.name
                    ),
                )
                .await?;
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(handled)
    }
}
