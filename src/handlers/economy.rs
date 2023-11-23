use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
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

impl DiscordCommandProvider for EconomyProvider {
    fn commands(&self) -> Vec<DiscordCommand> {
        vec![wallet()]
    }
}

#[poise::command(slash_command, prefix_command)]
async fn wallet(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    if let Some(guild_id) = ctx.guild_id() {
        let author_name = display_name_from_command_context(ctx).await;

        let db = &ctx.data().db;
        let wallet = Wallet::get_from_db(db, guild_id, ctx.author().id)
            .await
            .map_err(|e| DiscordCommandError {
                message: format!("error getting wallet from db: {e}"),
                command_identifier: "wallet".to_string(),
            })?;

        // send the wallet balance
        ctx.reply(format!(
            "hey {author_name}! you have **{}** coins in your wallet.",
            wallet.balance().to_formatted_string(&Locale::en)
        ))
        .await
        .map_err(|e| DiscordCommandError {
            message: format!("error sending message: {e}"),
            command_identifier: "wallet".to_string(),
        })?;

        Ok(())
    } else {
        ctx.say("this command can only be used in a server! each server has their own economy. use this command in a server you're in to check your balance there! ^w^")
            .await.map_err(|e| DiscordCommandError {
                message: format!("error sending message: {e}"),
                command_identifier: "wallet".to_string(),
            })?;

        Ok(())
    }
}
