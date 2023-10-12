use std::{error::Error, fmt::{Display, self}};

use crate::MuniBotError;

use super::DiscordState;

pub trait DiscordCommandProvider: Send {
    fn commands(&self) -> Vec<poise::Command<DiscordState, MuniBotError>>;
}

#[derive(Debug)]
pub struct DiscordCommandError {
    pub message: String,
    pub command_identifier: String,
}
impl Display for DiscordCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the `{}` discord command encountered an error: {}",
            self.command_identifier, self.message
        )
    }
}
impl Error for DiscordCommandError {}
