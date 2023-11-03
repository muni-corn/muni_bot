#![feature(decl_macro)]
#![feature(let_chains)]

use std::{fmt::Display, io::Cursor, sync::Arc};

use discord::{commands::DiscordCommandError, start_discord_integration};
use handlers::{magical::MagicalHandler, DiscordCommandProviderCollection};
use rocket::{http::ContentType, response::Responder, Response};
use tokio::sync::{mpsc::error::SendError, Mutex};
use twitch::{auth::state::TwitchAuthState, bot::TwitchBot};
use twitch_irc::login::UserAccessToken;

use crate::{
    handlers::{
        bot_affection::BotAffectionProvider, dice::DiceHandler, greeting::GreetingHandler,
        DiscordHandlerCollection,
    },
    twitch::auth::state::get_basic_url,
};

mod discord;

mod auth_server;
mod bot;
mod handlers;
mod schema;
mod twitch;

#[rocket::main]
async fn main() -> Result<(), MuniBotError> {
    dotenvy::dotenv().ok();

    // ensure credentials exist
    match std::env::var("TWITCH_TOKEN") {
        Ok(twitch_token) => {
            // start twitch
            let twitch_handle = TwitchBot::new().start("muni_corn".to_owned(), twitch_token);

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
            if let Err(e) = twitch_handle.await {
                eprintln!("twitch bot died with error: {e}");
            }

            // wait for the discord bot to stop, if ever
            if let Err(e) = discord_handle.await {
                eprintln!("discord bot died with error: {e}");
            }

            Ok(())
        }
        Err(e) => {
            // let (twitch_auth_state, _) = TwitchAuthState::new();
            let auth_page_url = get_basic_url();
            println!("no twitch token found ({e})");
            println!("visit {auth_page_url} to get a token");
            Err(MuniBotError::MissingToken)
        }
    }
}

#[derive(Debug)]
enum MuniBotError {
    StateMismatch { got: String },
    ParseError(String),
    RequestError(String),
    SendError(String),
    DiscordCommand(DiscordCommandError),
    MissingToken,
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

impl Display for MuniBotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuniBotError::StateMismatch { got } => write!(f, "state mismatch from twitch. be careful! this could mean someone is trying to do something malicious. (got code \"{got}\")"),
            MuniBotError::ParseError(e) => write!(f, "parsing failure! {e}"),
            MuniBotError::RequestError(e) => write!(f, "blegh!! {e}"),
            MuniBotError::SendError(e) => write!(f, "send error! {e}"),
            MuniBotError::DiscordCommand(e) => e.fmt(f),
            MuniBotError::MissingToken => write!(f, "missing token!"),
        }
    }
}

impl std::error::Error for MuniBotError {}

impl<'req> Responder<'req, 'static> for MuniBotError {
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'static> {
        let display = format!("{self}");
        Response::build()
            .header(ContentType::Plain)
            .sized_body(display.len(), Cursor::new(display))
            .ok()
    }
}

impl From<DiscordCommandError> for MuniBotError {
    fn from(e: DiscordCommandError) -> Self {
        Self::DiscordCommand(e)
    }
}
