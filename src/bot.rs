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

    /// Begin listening for auth tokens and start bots when they are received.
    pub async fn run(self) {
        let twitch_handle = Self::listen_twitch(self.twitch_auth_receiver);

        // block on twitch execution until it fails
        twitch_handle.await.unwrap();
    }

    /// Spawns a task for listening for Twitch auth tokens and starting Twitch bots.
    fn listen_twitch(mut auth_rx: Receiver<UserAccessToken>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut twitch_bot_handles = Vec::new();

            while let Some(token) = auth_rx.recv().await {
                // start a twitch bot with the token received and add its handle
                let handle = TwitchBot::new(token, "muni_corn").start().await;
                twitch_bot_handles.push(handle);
            }

            for handle in twitch_bot_handles {
                if let Err(e) = handle.await {
                    println!("twitch join handle error: {}", e);
                }
            }
        })
    }
}
