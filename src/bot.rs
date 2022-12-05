use twitch_irc::{
    login::{RefreshingLoginCredentials, UserAccessToken},
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
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                println!("Received message: {:?}", message);
            }
        });

        // join a channel
        // This function only returns an error if the passed channel login name is malformed,
        // so in this simple case where the channel name is hardcoded we can ignore the potential
        // error with `unwrap`.
        client.join("muni_corn".to_owned()).unwrap();

        // keep the tokio executor alive.
        // If you return instead of waiting the background task will exit.
        join_handle.await.unwrap();
    }
}
