use tokio::sync::Mutex;
use std::sync::Arc;

use rocket::{routes, Ignite, Rocket};
use tokio::task::JoinHandle;

use crate::twitch::auth::{
    routes::{catch_twitch_oauth_error, twitch_oauth_callback},
    state::TwitchAuthState,
    AuthTokenReceivers,
};

pub(crate) const REDIRECT_URI: &str = "http://localhost:6864/";

/// AuthServer is a server that handles authorization with various services.
pub struct AuthServer {
    twitch_auth_state: Arc<Mutex<TwitchAuthState>>,
}

impl AuthServer {
    pub fn new() -> (Self, AuthTokenReceivers) {
        let (twitch_auth, twitch_auth_rx) = TwitchAuthState::new();

        (
            Self {
                twitch_auth_state: Arc::new(Mutex::new(twitch_auth)),
            },
            AuthTokenReceivers {
                twitch: twitch_auth_rx,
            },
        )
    }

    /// This function does not block when awaited. It returns a JoinHandle that can be awaited
    /// to wait for the server to stop.
    pub async fn launch(
        &mut self,
    ) -> Result<JoinHandle<Result<Rocket<Ignite>, rocket::Error>>, rocket::Error> {
        let rocket = rocket::build()
            .manage(self.twitch_auth_state.clone())
            .mount(
                "/twitch",
                routes![twitch_oauth_callback, catch_twitch_oauth_error],
            )
            .ignite()
            .await?;

        Ok(tokio::task::spawn(async { rocket.launch().await }))
    }
}

/// Opens an autorization page with a new thread. open-rs is not supposed to block, but it
/// does anyways for some reason lol
#[must_use]
pub fn open_auth_page(auth_page_url: reqwest::Url) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        println!("opening authorization page");
        if let Err(e) = open::that(auth_page_url.to_string()) {
            eprintln!("couldn't open url: {e}");
            eprintln!("to authorize, open up this url: {auth_page_url}");
        } else {
            println!("opened auth page");
        }
    })
}
