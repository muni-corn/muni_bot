#![feature(decl_macro)]
#![feature(let_chains)]

use std::{error::Error, fmt::Display, io::Cursor, sync::Arc};

use discord::{commands::DiscordCommandError, start_discord_integration};
use handlers::{magical::MagicalHandler, DiscordCommandProviderCollection};
use rocket::{http::ContentType, response::Responder, Response};
use tokio::sync::{mpsc::error::SendError, Mutex};
use twitch_irc::login::UserAccessToken;

use crate::handlers::{
    dice::DiceHandler, greeting::GreetingHandler, bot_affection::BotAffectionProvider, DiscordHandlerCollection,
};

mod discord;

mod auth_server;
mod bot;
mod handlers;
mod schema;
mod twitch;

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let discord_handlers: DiscordHandlerCollection = vec![Arc::new(Mutex::new(GreetingHandler))];
    let discord_command_providers: DiscordCommandProviderCollection = vec![
        Box::new(DiceHandler),
        Box::new(BotAffectionProvider),
        Box::new(MagicalHandler),
    ];
    start_discord_integration(discord_handlers, discord_command_providers).await;

    // // open web browser to authorize
    // let twitch_auth_page_handle = open_auth_page(
    //     auth_server
    //     .get_twitch_auth_state()
    //     .lock()
    //     .await
    //     .get_auth_page_url().clone(),
    // );

    // let bot = MuniBot::new(auth_rxs);
    // bot.run().await;

    // // wait for the auth server to stop, if ever
    // let _ = auth_server_handle.await??;

    // // wait for the twitch auth page to close, if ever
    // twitch_auth_page_handle.await?;

    Ok(())
}

#[derive(Debug)]
enum MuniBotError {
    StateMismatch { got: String },
    ParseError(String),
    RequestError(String),
    SendError(String),
    DiscordCommand(DiscordCommandError),
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
