use async_trait::async_trait;
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage};

use crate::twitch::{
    agent::TwitchAgent,
    bot::MuniBotTwitchIRCClient,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

pub struct LurkHandler;

#[async_trait]
impl TwitchMessageHandler for LurkHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        _agent: &TwitchAgent<StaticLoginCredentials>,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!lurk") {
                self.send_twitch_message(
                    client,
                    &m.channel_login,
                    &format!("{} cast an invisibility spell!", m.sender.name),
                )
                .await?;
                true
            } else if m.message_text.trim().starts_with("!unlurk") {
                self.send_twitch_message(
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
