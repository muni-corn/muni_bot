use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use super::{HandlerError, MessageHandler};
use crate::bot::MuniBotTwitchIRCClient;

pub struct RaidMsgHandler;

#[async_trait]
impl MessageHandler for RaidMsgHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, HandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!rmsg") {
                self.send_message(
                    client,
                    &m.channel_login,
                    include_str!("../../raid_msg_normal.txt"),
                )
                .await?;

                self.send_message(
                    client,
                    &m.channel_login,
                    include_str!("../../raid_msg_subs.txt"),
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
