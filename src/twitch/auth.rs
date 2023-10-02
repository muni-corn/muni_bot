use tokio::sync::mpsc::Receiver;
use twitch_irc::login::UserAccessToken;

pub mod routes;
pub mod state;

pub struct AuthTokenReceivers {
    pub twitch: Receiver<UserAccessToken>,
    // pub discord: todo!()
}

impl AuthTokenReceivers {
    pub fn new(twitch: Receiver<UserAccessToken>) -> Self {
        Self { twitch }
    }

    pub fn get_twitch_rx(&self) -> &Receiver<UserAccessToken> {
        &self.twitch
    }
}

pub fn get_client_tokens() -> (String, String) {
    use std::env::var;

    let client_id = var("TWITCH_CLIENT_ID")
        .expect("TWITCH_CLIENT_ID env var wasn't provided :/")
        .trim()
        .to_string();
    let client_secret = var("TWITCH_CLIENT_SECRET")
        .expect("TWITCH_CLIENT_SECRET env var wasn't provided :/")
        .trim()
        .to_string();

    (client_id, client_secret)
}
