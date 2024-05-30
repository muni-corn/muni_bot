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

pub struct RaidMsgHandler;

#[async_trait]
impl TwitchMessageHandler for RaidMsgHandler {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        irc_client: &MuniBotTwitchIRCClient,
        _helix_client: &HelixClient<reqwest::Client>,
        _agent: &TwitchAgent<StaticLoginCredentials>,
        config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!rmsg") {
                if config.twitch.raid_msg_subs.is_none() && config.twitch.raid_msg_all.is_none() {
                    self.send_twitch_message(
                        irc_client,
                            &m.channel_login,
                            "the raid message command is enabled, but no raid messages have been configured >.>",
                        )
                        .await?;
                } else {
                    if let Some(raid_msg_all) = &config.twitch.raid_msg_all {
                        self.send_twitch_message(irc_client, &m.channel_login, raid_msg_all)
                            .await?;
                    }

                    if let Some(raid_msg_subs) = &config.twitch.raid_msg_subs {
                        self.send_twitch_message(irc_client, &m.channel_login, raid_msg_subs)
                            .await?;
                    }
                }

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
