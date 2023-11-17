use async_trait::async_trait;
use poise::serenity_prelude::{Context, Message};

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        handler::{DiscordMessageHandler, DiscordMessageHandlerError},
        utils::display_name_from_command_context,
        DiscordCommand, DiscordContext, DiscordFrameworkContext,
    },
    MuniBotError,
};

use wallet::Wallet;

mod wallet;

pub struct EconomyProvider;

impl EconomyProvider {
    fn calc_salary(msg: &Message) -> u64 {
        // determine a salary based on words
        msg.content
            .split_whitespace()
            .filter_map(|w| {
                // ignore words containing symbols
                if w.contains(|c: char| !c.is_alphanumeric()) {
                    None
                } else if w.len() > 2 {
                    // pay one unit per character, up to 10 units per word,
                    // and only for words greater than 2 characters
                    Some(w.len().min(10) as u64)
                } else {
                    None
                }
            })
            .sum()
    }
}

#[async_trait]
impl DiscordMessageHandler for EconomyProvider {
    fn name(&self) -> &'static str {
        "economy"
    }

    async fn handle_discord_message(
        &mut self,
        _context: &Context,
        framework: DiscordFrameworkContext<'_>,
        msg: &Message,
    ) -> Result<bool, DiscordMessageHandlerError> {
        if let Some(guild_id) = msg.guild_id {
            let salary = Self::calc_salary(msg);
            let db = &framework.user_data().await.db;

            Wallet::get_from_db(db, guild_id, msg.author.id)
                .await
                .map_err(|e| DiscordMessageHandlerError {
                    handler_name: self.name(),
                    message: format!("error getting wallet from db: {e}"),
                })?
                .deposit(db, salary)
                .await
                .map_err(|e| DiscordMessageHandlerError {
                    handler_name: self.name(),
                    message: format!("error depositing salary: {e}"),
                })?;
        }

        // return false to allow subsequent handlers to handle this message
        Ok(false)
    }
}

