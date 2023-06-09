use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use super::{HandlerError, MessageHandler};
use crate::bot::MuniBotTwitchIRCClient;

pub struct SocialsHandler;

#[async_trait]
impl MessageHandler for SocialsHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, HandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!discord") {
                self.send_message(
                    client,
                    &m.channel_login,
                    &format!(
                        "join the herd's discord server here! {} we have treats:)",
                        include_str!("../../discord_link.txt")
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
