use tokio::{task::JoinHandle, sync::mpsc::Receiver};
use twitch_irc::login::UserAccessToken;

use crate::twitch::{auth::AuthTokenReceivers, bot::TwitchBot};

pub struct MuniBot {
    twitch_auth_receiver: Receiver<UserAccessToken>,
}

impl MuniBot {
    pub fn new(auth_token_receivers: AuthTokenReceivers) -> MuniBot {
        Self {
            twitch_auth_receiver: auth_token_receivers.twitch,
        }
    }
}
