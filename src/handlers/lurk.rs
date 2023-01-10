use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use super::{MessageHandler, HandlerError};
use crate::bot::MuniBotTwitchIRCClient;

pub struct LurkHandler;

#[async_trait]
impl MessageHandler for LurkHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, HandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!lurk") {
                if let Err(e) = client
                    .say(
                        m.channel_login.clone(),
                        format!("{} cast an invisibility spell!", m.sender.name),
                    )
                    .await
                {
                    eprintln!("message send failure! {e}")
                }
                true
            } else if m.message_text.trim().starts_with("!unlurk") {
                if let Err(e) = client
                    .say(
                        m.channel_login.clone(),
                        format!(
                            "{}'s invisibility spell wore off. we can see you!",
                            m.sender.name
                        ),
                    )
                    .await
                {
                    eprintln!("message send failure! {e}")
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
