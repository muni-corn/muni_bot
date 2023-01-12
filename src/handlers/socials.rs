use async_trait::async_trait;
use twitch_irc::message::ServerMessage;

use super::MessageHandler;
use crate::bot::MuniBotTwitchIRCClient;

pub struct SocialsHandler;

#[async_trait]
impl MessageHandler for SocialsHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> bool {
        if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("!discord") {
                if let Err(e) = client
                    .say(
                        m.channel_login.clone(),
                        format!(
                            "join the herd's discord server here! {} (we have treats :))",
                            include_str!("../../discord_link.txt")
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
