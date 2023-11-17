#![feature(decl_macro)]
#![feature(let_chains)]
#![feature(async_fn_in_trait)]

use std::sync::Arc;

use discord::{commands::DiscordCommandError, start_discord_integration};
use handlers::{magical::MagicalHandler, DiscordCommandProviderCollection};

use poise::serenity_prelude as serenity;
use thiserror::Error;
use tokio::sync::{mpsc::error::SendError, Mutex};
use twitch::bot::TwitchBot;
use twitch_irc::login::UserAccessToken;

use crate::{
    handlers::{
        bot_affection::BotAffectionProvider, dice::DiceHandler, economy::EconomyProvider,
        eight_ball::EightBallProvider, greeting::GreetingHandler,
        ventriloquize::VentriloquizeProvider, DiscordHandlerCollection,
    },
    twitch::get_basic_auth_url,
};

mod discord;

mod handlers;
mod twitch;

#[tokio::main]
async fn main() -> Result<(), MuniBotError> {
    dotenvy::dotenv().ok();

    // ensure credentials exist
    match std::env::var("TWITCH_TOKEN") {
        Ok(twitch_token) => {
            // start twitch
            let twitch_handle = TwitchBot::new()
                .await
                .start("muni_corn".to_owned(), twitch_token);

            // start discord
            let discord_handlers: DiscordHandlerCollection = vec![
                Arc::new(Mutex::new(GreetingHandler)),
                Arc::new(Mutex::new(EconomyProvider)),
            ];
            let discord_command_providers: DiscordCommandProviderCollection = vec![
                Box::new(DiceHandler),
                Box::new(BotAffectionProvider),
                Box::new(MagicalHandler),
                Box::new(EightBallProvider),
                Box::new(VentriloquizeProvider),
            ];
            let discord_handle = tokio::spawn(start_discord_integration(
                discord_handlers,
                discord_command_providers,
            ));

            // wait for the twitch bot to stop, if ever
            match twitch_handle.await {
                Ok(_) => println!("twitch bot stopped o.o"),
                Err(e) => eprintln!("twitch bot died with error: {e}"),
            }

            // wait for the discord bot to stop, if ever
            match discord_handle.await {
                Ok(_) => println!("discord bot stopped o.o"),
                Err(e) => eprintln!("discord bot died with error: {e}"),
            }

            println!("all bot integrations have stopped. goodbye ^-^");
            Ok(())
        }
        Err(e) => {
            let auth_page_url = get_basic_auth_url();
            println!("no TWITCH_TOKEN found ({e})");
            println!("visit {auth_page_url} to get a token");
            Err(MuniBotError::MissingToken)
        }
    }
}

#[derive(Error, Debug)]
enum MuniBotError {
    #[error("parsing failure :< {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("request failed :< {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("token send failed :< {0}")]
    SendError(#[from] SendError<UserAccessToken>),

    #[error("discord `{0}` command failed :< `{1}`")]
    DiscordCommand(String, String),

    #[error("missing token :<")]
    MissingToken,

    #[error("error with database :< {0}")]
    DbError(#[from] surrealdb::Error),

    #[error("caught a serenity error: {0}")]
    SerenityError(#[from] serenity::Error),

    #[error("something different went wrong :< {0}")]
    Other(String),
}

impl From<DiscordCommandError> for MuniBotError {
    fn from(e: DiscordCommandError) -> Self {
        Self::DiscordCommand(e.command_identifier.to_string(), format!("{e}"))
    }
}
