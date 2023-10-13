use async_trait::async_trait;
use chrono::Local;
use twitch_irc::message::ServerMessage;
use xxhash_rust::const_xxh3::xxh3_64;

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        DiscordState,
    },
    twitch::{
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
    MuniBotError,
};

pub struct MagicalHandler;

impl MagicalHandler {
    fn get_magic_amount(user_id: &str) -> u8 {
        let date_user_id = format!("{}{user_id}", Local::now().date_naive());
        let hashed = xxh3_64(date_user_id.as_bytes());
        let x = 1.0 - ((hashed % 100 + 1) as f32 / 100.0);
        ((1.0 - x * x * x) * 100.0) as u8
    }

    fn get_message(user_id: &str, user_display_name: &str) -> String {
        let magic_amount = Self::get_magic_amount(user_id);
        let suffix = match magic_amount {
            1 => ". ouch. lol.",
            100 => "!! wow :3",
            x if x < 25 => ". sounds like a good day for some self care. <3",
            _ => "!",
        };
        format!("{user_display_name} is {magic_amount}% magical today{suffix}")
    }
}

#[async_trait]
impl TwitchMessageHandler for MagicalHandler {
    async fn handle_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: &ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = match message {
            ServerMessage::Privmsg(msg) => {
                if msg.message_text.starts_with("!magical") {
                    self.send_twitch_message(
                        client,
                        &msg.channel_login,
                        &Self::get_message(&msg.sender.id, &msg.sender.name),
                    )
                    .await?;
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        Ok(handled)
    }
}

/// Check your magicalness today.
#[poise::command(prefix_command, slash_command)]
async fn magical(ctx: poise::Context<'_, DiscordState, MuniBotError>) -> Result<(), MuniBotError> {
    ctx.say(MagicalHandler::get_message(
        &ctx.author().id.to_string(),
        &ctx.author().name,
    ))
    .await
    .map_err(|e| DiscordCommandError {
        message: format!("couldn't send message: {}", e),
        command_identifier: "magical".to_string(),
    })?;

    Ok(())
}

impl DiscordCommandProvider for MagicalHandler {
    fn commands(&self) -> Vec<poise::Command<DiscordState, MuniBotError>> {
        vec![magical()]
    }
}
