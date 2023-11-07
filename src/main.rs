#![feature(decl_macro)]
#![feature(let_chains)]

use std::{fmt::Display, sync::Arc};

use discord::{commands::DiscordCommandError, start_discord_integration};
use handlers::{magical::MagicalHandler, DiscordCommandProviderCollection};
use tokio::sync::{mpsc::error::SendError, Mutex};
use twitch::bot::TwitchBot;
use twitch_irc::login::UserAccessToken;

use crate::{
    handlers::{
        bot_affection::BotAffectionProvider, dice::DiceHandler, greeting::GreetingHandler,
        DiscordHandlerCollection,
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
            let twitch_handle = TwitchBot::new(twitch_token)
                .await
                .start("muni_corn".to_owned());

            // start discord
            let discord_handlers: DiscordHandlerCollection =
                vec![Arc::new(Mutex::new(GreetingHandler))];
            let discord_command_providers: DiscordCommandProviderCollection = vec![
                Box::new(DiceHandler),
                Box::new(BotAffectionProvider),
                Box::new(MagicalHandler),
            ];
            let discord_handle = tokio::spawn(start_discord_integration(
                discord_handlers,
                discord_command_providers,
            ));

            // wait for the twitch bot to stop, if ever
            match twitch_handle.await {
                Ok(_) => println!("twitch bot stopped o.o"),
                Err(e) => eprintln!("twitch bot died with error: {e}")
            }

            // wait for the discord bot to stop, if ever
            match discord_handle.await {
                Ok(_) => println!("discord bot stopped o.o"),
                Err(e) => eprintln!("discord bot died with error: {e}")
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

#[derive(Debug)]
enum MuniBotError {
    ParseError(String),
    RequestError(String),
    SendError(String),
    DiscordCommand(DiscordCommandError),
    MissingToken,
    DbError(surrealdb::Error),
    Other(String),
}

impl From<serde_json::Error> for MuniBotError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(format!("couldn't parse: {e}"))
    }
}

impl From<reqwest::Error> for MuniBotError {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestError(format!("request failed: {e}"))
    }
}

impl From<SendError<UserAccessToken>> for MuniBotError {
    fn from(e: SendError<UserAccessToken>) -> Self {
        Self::SendError(format!("sending token failed: {e}"))
    }
}

impl From<DiscordCommandError> for MuniBotError {
    fn from(e: DiscordCommandError) -> Self {
        Self::DiscordCommand(e)
    }
}

impl From<surrealdb::Error> for MuniBotError {
    fn from(value: surrealdb::Error) -> Self {
        Self::DbError(value)
    }
}

impl Display for MuniBotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuniBotError::ParseError(e) => write!(f, "parsing failure! {e}"),
            MuniBotError::RequestError(e) => write!(f, "blegh!! {e}"),
            MuniBotError::SendError(e) => write!(f, "send error! {e}"),
            MuniBotError::DiscordCommand(e) => e.fmt(f),
            MuniBotError::MissingToken => write!(f, "missing token!"),
            MuniBotError::DbError(e) => write!(f, "database error :( {e}"),
            MuniBotError::Other(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for MuniBotError {}
