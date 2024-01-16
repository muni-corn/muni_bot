use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    discord::{commands::DiscordCommandProvider, handler::DiscordMessageHandler},
    twitch::handler::TwitchMessageHandler,
};

pub mod affection;
pub mod bonk;
pub mod bot_affection;
pub mod content_warning;
pub mod dice;
pub mod economy;
pub mod eight_ball;
pub mod greeting;
pub mod lift;
pub mod lurk;
pub mod magical;
pub mod quotes;
pub mod raid_msg;
pub mod shoutout;
pub mod socials;
pub mod temperature;
pub mod topic_change;
pub mod ventriloquize;

pub type TwitchHandlerCollection = Vec<Arc<Mutex<dyn TwitchMessageHandler>>>;
pub type DiscordHandlerCollection = Vec<Arc<Mutex<dyn DiscordMessageHandler>>>;
pub type DiscordCommandProviderCollection = Vec<Box<dyn DiscordCommandProvider>>;
