use async_trait::async_trait;
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage};

use crate::twitch::{
    agent::TwitchAgent,
    bot::MuniBotTwitchIRCClient,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

#[derive(Default)]
pub struct AffectionHandler;

#[async_trait]
impl TwitchMessageHandler for AffectionHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        _agent: &TwitchAgent<StaticLoginCredentials>,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if let Some(target) = m.message_text.trim().strip_prefix("!hug ") {
                self.send_twitch_message(
                    client,
                    &m.channel_login,
                    &format!("{} gets the biggest huggle from {}!", target, m.sender.name),
                )
                .await?;
                true
            } else if let Some(target) = m.message_text.trim().strip_prefix("!glomp ") {
                self.send_twitch_message(
                    client,
                    &m.channel_login,
                    &format!("{} tackle hugs {}! o.o", target, m.sender.name),
                )
                .await?;
                true
            } else if let Some(target) = m.message_text.trim().strip_prefix("!nuzzle ") {
                self.send_twitch_message(
                    client,
                    &m.channel_login,
                    &format!("{} nuzzle wuzzles {}~", m.sender.name, target),
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
