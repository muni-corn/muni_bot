use async_trait::async_trait;
use lazy_static::lazy_static;
use poise::serenity_prelude::{Context, Message};
use rand::Rng;
use regex::Regex;
use twitch_irc::message::ServerMessage;

use crate::{
    discord::handler::{DiscordMessageHandler, DiscordMessageHandlerError},
    twitch::{
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
};

pub struct GreetingHandler;

lazy_static! {
    static ref HI_REGEX: Regex =
        Regex::new(r"(?i)(?:hi+|hey+|hello+|howdy|sup).*muni.*bot").unwrap();
}

impl GreetingHandler {
    /// Returns a greeting message if applicable, or None if not to keep quiet.
    fn get_greeting_message(user_name: &str, message_text: &str) -> Option<String> {
        if message_text.trim().starts_with("uwu") {
            if user_name == "Linokii" {
                Some("linokii uwu<3".to_string())
            } else {
                None
            }
        } else if HI_REGEX.is_match(message_text) {
            // send a hi message back
            // pick a template
            let template_index = rand::thread_rng().gen_range(0..HELLO_TEMPLATES.len());
            let mut greeting = HELLO_TEMPLATES[template_index].replace("{name}", user_name);

            // if the message was sent from linokii, append a very special uwu
            if user_name == "Linokii" {
                greeting.push_str(" uwu");
            }

            Some(greeting)
        } else {
            None
        }
    }
}

#[async_trait]
impl TwitchMessageHandler for GreetingHandler {
    async fn handle_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: &ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if let Some(response) = Self::get_greeting_message(&m.sender.name, &m.message_text) {
                self.send_message(client, &m.channel_login, &response)
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

#[async_trait]
impl DiscordMessageHandler for GreetingHandler {
    fn name(&self) -> &'static str {
        "greeting"
    }

    async fn handle_message(
        &mut self,
        context: &Context,
        msg: &Message,
    ) -> Result<bool, DiscordMessageHandlerError> {
        let handled =
            if let Some(response) = Self::get_greeting_message(&msg.author.name, &msg.content) {
                msg.channel_id
                    .say(&context.http, response)
                    .await
                    .map_err(|e| DiscordMessageHandlerError {
                        message: e.to_string(),
                        handler_name: self.name().to_string(),
                    })?;
                true
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
