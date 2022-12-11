use std::sync::Arc;

use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use twitch_irc::{
    login::{RefreshingLoginCredentials, UserAccessToken},
    message::ServerMessage,
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::token_storage::MuniBotTokenStorage;

pub struct MuniBot {
    user_access_token: UserAccessToken,
}

impl MuniBot {
    pub fn new(user_access_token: UserAccessToken) -> Self {
        Self { user_access_token }
    }

    pub async fn run(self) {
        let client_id = include_str!("./client_id.txt").trim().to_owned();
        let client_secret = include_str!("./client_secret.txt").to_owned();
        let token_storage = MuniBotTokenStorage {
            user_access_token: self.user_access_token,
        };
        let credentials = RefreshingLoginCredentials::init(client_id, client_secret, token_storage);
        let config = ClientConfig::new_simple(credentials);

        let (mut incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, _>::new(config);

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let client = Arc::new(client);
        let client_clone = client.clone();
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                Self::handle_message(&client_clone, message).await;
            }
        });

        // join a channel
        // This function only returns an error if the passed channel login name is malformed,
        // so in this simple case where the channel name is hardcoded we can ignore the potential
        // error with `unwrap`.
        client.join("muni_corn".to_owned()).unwrap();

        client
            .say("muni_corn".to_string(), "i'm here!".to_owned())
            .await
            .unwrap();

        // keep the tokio executor alive.
        // If you return instead of waiting the background task will exit.
        join_handle.await.unwrap();
    }

    async fn handle_message(
        client: &TwitchIRCClient<
            SecureTCPTransport,
            RefreshingLoginCredentials<MuniBotTokenStorage>,
        >,
        message: ServerMessage,
    ) {
        match message {
            ServerMessage::Privmsg(m) => {
            }
            m => eprintln!("unhandled message: {:#?}", m),
        }
    }
}
