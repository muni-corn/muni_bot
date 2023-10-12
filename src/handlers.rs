use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    discord::{commands::DiscordCommandProvider, handler::DiscordMessageHandler},
    twitch::handler::TwitchMessageHandler,
};

pub mod bonk;
pub mod content_warning;
pub mod dice;
pub mod greeting;
pub mod lurk;
pub mod raid_msg;
pub mod socials;

pub type TwitchHandlerCollection = Vec<Arc<Mutex<dyn TwitchMessageHandler>>>;
pub type DiscordHandlerCollection = Vec<Arc<Mutex<dyn DiscordMessageHandler>>>;
pub type DiscordCommandProviderCollection = Vec<Box<dyn DiscordCommandProvider>>;
