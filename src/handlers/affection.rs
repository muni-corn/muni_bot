use async_trait::async_trait;
use twitch_api::HelixClient;
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage};

use crate::{
    config::Config,
    twitch::{
        agent::TwitchAgent,
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
};

#[derive(Default)]
pub struct AffectionHandler;

#[async_trait]
impl TwitchMessageHandler for AffectionHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        irc_client: &MuniBotTwitchIRCClient,
        _helix_client: &HelixClient<reqwest::Client>,
        _agent: &TwitchAgent<StaticLoginCredentials>,
        _config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            let message_text = m.message_text.trim();
            if let Some(target) = message_text.strip_prefix("!hug ") {
                self.send_twitch_message(
                    irc_client,
                    &m.channel_login,
                    &format!("{} gets the biggest huggle from {}!", target, m.sender.name),
                )
                .await?;
                true
            } else if let Some(target) = message_text.strip_prefix("!glomp ") {
                self.send_twitch_message(
                    irc_client,
                    &m.channel_login,
                    &format!("{} tackle hugs {}! o.o", target, m.sender.name),
                )
                .await?;
                true
            } else if let Some(target) = message_text.strip_prefix("!nuzzle ") {
                self.send_twitch_message(
                    irc_client,
                    &m.channel_login,
                    &format!("{} nuzzle wuzzles {}~", m.sender.name, target),
                )
                .await?;
                true
            } else if let Some(target) = message_text.strip_prefix("!boop ") {
                self.send_twitch_message(
                    irc_client,
                    &m.channel_login,
                    &format!("{} has been booped by {}!", target, m.sender.name),
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
