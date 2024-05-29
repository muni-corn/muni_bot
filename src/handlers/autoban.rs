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
        if let ServerMessage::Join(join_msg) = message {
            if join_msg.user_login.starts_with("brandon") {
                let Some(broadcaster_id) =
                    agent.get_user_from_login(&join_msg.channel_login).await?
                else {
                    return Err(TwitchHandlerError::Other(format!(
                        "could not get broadcaster id for channel {}",
                        join_msg.channel_login
                    )));
                };

                agent
                    .ban_user(
                        &join_msg.user_login,
                        "user is suspected to be an alt of brandontheponybrony",
                        &broadcaster_id.id,
                    )
                    .await?;
            }
        }
        Ok(true)
    }
}
