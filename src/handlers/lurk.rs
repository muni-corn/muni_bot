use async_trait::async_trait;
use twitch_irc::{
    login::RefreshingLoginCredentials, message::ServerMessage, SecureTCPTransport, TwitchIRCClient,
};

use crate::token_storage::MuniBotTokenStorage;

use super::MessageHandler;

pub struct LurkHandler;

#[async_trait]
impl MessageHandler for LurkHandler {
    async fn handle_message(
        &mut self,
        client: &TwitchIRCClient<
            SecureTCPTransport,
            RefreshingLoginCredentials<MuniBotTokenStorage>,
        >,
        message: ServerMessage,
    ) -> bool {
        if let ServerMessage::Privmsg(m) = message {
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
        }
    }
}
