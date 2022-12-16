use async_trait::async_trait;
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use twitch_irc::message::ServerMessage;

use crate::bot::MuniBotTwitchIRCClient;
use super::MessageHandler;

pub struct GreetingHandler;

#[async_trait]
impl MessageHandler for GreetingHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> bool {
        lazy_static! {
            static ref HI_REGEX: Regex =
                Regex::new(r"(?i)(?:hi+|hey+|hello+|howdy).*muni.*bot").unwrap();
        }

        if let ServerMessage::Privmsg(m) = message {
            if m.message_text.trim().starts_with("uwu") {
                if m.sender.name == "Linokii" {
                    if let Err(e) = client
                        .say(m.channel_login.clone(), "linokii uwu<3".to_string())
                        .await
                    {
                        eprintln!("message send failure! {e}")
                    }
                } else if let Err(e) = client
                    .say(
                        m.channel_login.clone(),
                        format!(
                            "sorry {}, uwu is reserved for the one and only Linokii",
                            m.sender.name
                        ),
                    )
                    .await
                {
                    eprintln!("message send failure! {e}")
                }
                true
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

                if let Err(e) = client.say(m.channel_login, greeting).await {
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
