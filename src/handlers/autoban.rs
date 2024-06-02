use twitch_irc::message::ServerMessage;

use crate::{
    config::Config,
    twitch::{
        agent::TwitchAgent,
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
};

pub struct AutoBanHandler;

#[async_trait::async_trait]
impl TwitchMessageHandler for AutoBanHandler {
    /// Handle a new message from chat. Returns `true` if something was done to
    /// handle the message, or `false` if the message was ignored (or if the
    /// message is allowed to also be handled by other handlers).
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        _client: &MuniBotTwitchIRCClient,
        agent: &TwitchAgent,
        _config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        let (user_login, channel_login) = match message {
            ServerMessage::Join(join_msg) => (
                join_msg.user_login.as_str(),
                join_msg.channel_login.as_str(),
            ),
            ServerMessage::Privmsg(privmsg) => (
                privmsg.sender.login.as_str(),
                privmsg.channel_login.as_str(),
            ),
            _ => return Ok(false),
        };

        if user_login.starts_with("brandon") || user_login.starts_with("phoenixredtailis") {
            let Some(broadcaster_id) = agent.get_user_from_login(channel_login).await? else {
                return Err(TwitchHandlerError::Other(format!(
                    "could not get broadcaster id for channel {}",
                    channel_login
                )));
            };

            let Some(ban_user_id) = agent.get_user_from_login(user_login).await? else {
                return Err(TwitchHandlerError::Other(format!(
                    "could not get user id for user {} on channel {}",
                    user_login, channel_login
                )));
            };

            agent
                .ban_user(
                    &ban_user_id.id,
                    "user is suspected to be an alt of brandontheponybrony",
                    &broadcaster_id.id,
                )
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
