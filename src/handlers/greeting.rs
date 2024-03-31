use async_trait::async_trait;
use once_cell::sync::Lazy;
use poise::serenity_prelude::{Context, Message};
use rand::seq::SliceRandom;
use regex::Regex;
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage};

use crate::{
    config::Config,
    discord::{
        handler::{DiscordMessageHandler, DiscordMessageHandlerError},
        utils::display_name_from_message,
        DiscordFrameworkContext,
    },
    twitch::{
        agent::TwitchAgent,
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
};

pub struct GreetingHandler;

static HI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:hi+|hey+|hello+|howdy+|sup+|heww?o+|henlo+)\b.*\bmuni.?bot\b").unwrap()
});

impl GreetingHandler {
    /// Returns a greeting message if applicable, or None if not to keep quiet.
    fn get_greeting_message(user_name: &str, message_text: &str) -> Option<String> {
        if message_text.trim().starts_with("uwu") {
            if user_name.to_lowercase() == "linokii" {
                Some("linokii uwu<3".to_string())
            } else {
                None
            }
        } else if HI_REGEX.is_match(message_text) {
            // send a hi message back
            // pick a template
            let mut rng = rand::thread_rng();
            let mut greeting = HELLO_TEMPLATES
                .choose(&mut rng)
                .unwrap()
                .replace("{name}", user_name);

            // if the message was sent from linokii, append a very special uwu
            if user_name.to_lowercase() == "linokii" {
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
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        _agent: &TwitchAgent<StaticLoginCredentials>,
        _config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if let Some(response) = Self::get_greeting_message(&m.sender.name, &m.message_text) {
                self.send_twitch_message(client, &m.channel_login, &response)
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

    async fn handle_discord_message(
        &mut self,
        context: &Context,
        _framework: DiscordFrameworkContext<'_>,
        msg: &Message,
    ) -> Result<bool, DiscordMessageHandlerError> {
        let author_name = display_name_from_message(msg, &context.http).await;

        let handled = if let Some(response) = Self::get_greeting_message(&author_name, &msg.content)
            && msg.author.id != context.cache.current_user().id
        {
            msg.channel_id
                .say(&context.http, response)
                .await
                .map_err(|e| DiscordMessageHandlerError {
                    message: e.to_string(),
                    handler_name: self.name(),
                })?;
            true
        } else {
            false
        };

        Ok(handled)
    }
}

const HELLO_TEMPLATES: [&str; 11] = [
    "hi, {name}!<3",
    "hello, {name}! happy to see you!",
    "hey {name}:)",
    "hi {name}!! how are ya?",
    "{name}!! how are you doing?",
    "heyyy {name} uwu",
    "hi {name}! it's good to see you! :3",
    "{name} helloooooo:)",
    "hiiiii {name}",
    "hi {name}<3",
    "hi {name}! you look wonderful today ;3",
];
