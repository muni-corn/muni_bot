use async_trait::async_trait;
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage};

use crate::twitch::{
    agent::TwitchAgent,
    bot::MuniBotTwitchIRCClient,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

pub struct SocialsHandler;

#[async_trait]
impl TwitchMessageHandler for SocialsHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        _agent: &TwitchAgent<StaticLoginCredentials>,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!discord") {
                self.send_twitch_message(
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
