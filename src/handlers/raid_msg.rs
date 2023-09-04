use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use crate::twitch::{
    bot::MuniBotTwitchIRCClient,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

pub struct RaidMsgHandler;

#[async_trait]
impl TwitchMessageHandler for RaidMsgHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!rmsg") {
                self.send_twitch_message(
                    client,
                    &m.channel_login,
                    include_str!("../../raid_msg_normal.txt"),
                )
                .await?;

                self.send_twitch_message(
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
