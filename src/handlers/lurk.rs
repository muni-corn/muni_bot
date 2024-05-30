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

pub struct LurkHandler;

#[async_trait]
impl TwitchMessageHandler for LurkHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        irc_client: &MuniBotTwitchIRCClient,
        _helix_client: &HelixClient<reqwest::Client>,
        _agent: &TwitchAgent<StaticLoginCredentials>,
        _config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!lurk") {
                self.send_twitch_message(
                    irc_client,
                    &m.channel_login,
                    &format!("{} cast an invisibility spell!", m.sender.name),
                )
                .await?;
                true
            } else if m.message_text.trim().starts_with("!unlurk") {
                self.send_twitch_message(
                    irc_client,
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
