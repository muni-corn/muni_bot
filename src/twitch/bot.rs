use std::sync::Arc;

use async_trait::async_trait;
use twitch_irc::{
    login::{RefreshingLoginCredentials, UserAccessToken},
    message::ServerMessage,
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::handlers::{
    bonk::BonkHandler, greeting::GreetingHandler, lurk::LurkHandler, raid_msg::RaidMsgHandler,
    socials::SocialsHandler,
};

use super::{
    handler::{TwitchHandlerError, TwitchMessageHandler},
    token_storage::TwitchTokenStorage,
};

pub type MuniBotTwitchIRCClient =
    TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<TwitchTokenStorage>>;
pub type MuniBotTwitchIRCError =
    twitch_irc::Error<SecureTCPTransport, RefreshingLoginCredentials<TwitchTokenStorage>>;

pub struct TwitchBot {
    user_access_token: UserAccessToken,
    message_handlers: Vec<Box<dyn TwitchMessageHandler>>,
}

impl TwitchBot {
    pub fn new(user_access_token: UserAccessToken) -> Self {
        Self {
            user_access_token,
            message_handlers: vec![
                Box::new(BonkHandler),
                Box::new(SocialsHandler),
                Box::new(RaidMsgHandler),
                Box::new(LurkHandler),
                Box::new(GreetingHandler),
            ],
        }
    }

    pub async fn run(mut self) {
        let client_id = include_str!("../client_id.txt").trim().to_owned();
        let client_secret = include_str!("../client_secret.txt").to_owned();
        let token_storage = TwitchTokenStorage {
            user_access_token: self.user_access_token.clone(),
        };
        let credentials = RefreshingLoginCredentials::init(client_id, client_secret, token_storage);
        let config = ClientConfig::new_simple(credentials);

        let (mut incoming_messages, client) = MuniBotTwitchIRCClient::new(config);

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let client = Arc::new(client);
        let client_clone = client.clone();
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let Err(e) = self.handle_twitch_message(&client_clone, &message).await {
                    eprintln!("error in message handler! {e}");
                }
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
}

#[async_trait]
impl TwitchMessageHandler for TwitchBot {
    async fn handle_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: &ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        for message_handler in self.message_handlers.iter_mut() {
            // try to handle the message. if the handler determines the message was handled, we'll
            // stop
            if message_handler
                .handle_twitch_message(client, message)
                .await?
            {
                return Ok(true);
            }
        }

        return Ok(false);
    }
}