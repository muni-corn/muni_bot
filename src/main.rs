#![feature(decl_macro)]
#![feature(let_chains)]
#![feature(never_type)]

use std::sync::Arc;

use clap::Parser;
use config::Config;
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
        temperature::TemperatureConversionProvider, topic_change::TopicChangeProvider,
        ventriloquize::VentriloquizeProvider, DiscordHandlerCollection,
    },
    twitch::get_basic_auth_url,
};

mod config;
mod discord;
mod handlers;
mod twitch;

#[derive(Parser, Debug)]
struct Args {
    /// Path to a config file.
    #[clap(short, long, default_value = "/etc/muni_bot/config.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), MuniBotError> {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let config = Config::read_or_write_default_from(&args.config_file)?;

    // ensure credentials exist
    match std::env::var("TWITCH_TOKEN") {
        Ok(twitch_token) => {
            let discord_handle = start_discord(config.clone());

            // start twitch
            match TwitchBot::new(config.clone())
                .await
                .start("muni_corn".to_owned(), twitch_token, &config)
            {
                // wait for the twitch bot to stop, if ever
                Ok(twitch_handle) => match twitch_handle.await {
                    Ok(_) => println!("twitch bot stopped o.o"),
                    Err(e) => eprintln!("twitch bot died with error: {e}"),
                },
                Err(e) => eprintln!("twitch bot failed to start :< {e}"),
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

fn start_discord(config: Config) -> tokio::task::JoinHandle<()> {
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
        Box::new(EconomyProvider),
        Box::new(TopicChangeProvider),
        Box::new(TemperatureConversionProvider),
    ];

    tokio::spawn(start_discord_integration(
        discord_handlers,
        discord_command_providers,
        config,
    ))
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

    #[error("error in discord framework :< {0}")]
    SerenityError(#[from] serenity::Error),

    #[error("error loading config :< {0}, {1}")]
    LoadConfig(String, anyhow::Error),

    #[error("something different went wrong :< {0}")]
    Other(String),
}

impl From<DiscordCommandError> for MuniBotError {
    fn from(e: DiscordCommandError) -> Self {
        Self::DiscordCommand(e.command_identifier.to_string(), format!("{e}"))
    }
}
