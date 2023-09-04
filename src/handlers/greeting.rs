use async_trait::async_trait;
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use twitch_irc::message::ServerMessage;

use crate::twitch::{
    bot::MuniBotTwitchIRCClient,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

pub struct GreetingHandler;

#[async_trait]
impl TwitchMessageHandler for GreetingHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        lazy_static! {
            static ref HI_REGEX: Regex =
                Regex::new(r"(?i)(?:hi+|hey+|hello+|howdy|sup).*muni.*bot").unwrap();
        }

        let handled = if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("uwu") {
                if m.sender.name == "Linokii" {
                    self.send_message(client, &m.channel_login, "linokii uwu<3")
                        .await?;
                    true
                } else {
                    false
                }
            } else if HI_REGEX.is_match(&m.message_text) {
                // send a hi message back
                // pick a template
                let template_index = rand::thread_rng().gen_range(0..HELLO_TEMPLATES.len());
                let mut greeting =
                    HELLO_TEMPLATES[template_index].replace("{name}", &m.sender.name);

                // if the message was sent from linokii, append a very special uwu
                if m.sender.name == "Linokii" {
                    greeting.push_str(" uwu");
                }

                self.send_message(client, &m.channel_login, &greeting)
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

const HELLO_TEMPLATES: [&str; 10] = [
    "hi, {name}!<3",
    "hello, {name}! happy to see you!",
    "hey {name}:)",
    "hi {name}!! how are you?",
    "{name}!! how are you doing?",
    "heyyy {name} uwu",
    "hi {name}! it's good to see you!",
    "{name} helloooooo:)",
    "hiiiii {name}",
    "hi {name}<3",
];
