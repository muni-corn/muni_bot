use std::sync::Arc;

use async_trait::async_trait;
use tokio::task::JoinHandle;
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
    token_storage::TwitchTokenStorage, auth::get_client_tokens,
};

pub type MuniBotTwitchIRCClient =
    TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<TwitchTokenStorage>>;
pub type MuniBotTwitchIRCError =
    twitch_irc::Error<SecureTCPTransport, RefreshingLoginCredentials<TwitchTokenStorage>>;

pub struct TwitchBot {
    user_access_token: UserAccessToken,
    channel: String,
    message_handlers: Vec<Box<dyn TwitchMessageHandler>>,
}

impl TwitchBot {
    pub fn new(user_access_token: UserAccessToken, channel: &str) -> Self {
        Self {
            user_access_token,
            channel: channel.to_owned(),
            message_handlers: vec![
                Box::new(BonkHandler),
                Box::new(SocialsHandler),
                Box::new(RaidMsgHandler),
                Box::new(LurkHandler),
                Box::new(GreetingHandler),
            ],
        }
    }

    pub async fn start(mut self) -> JoinHandle<()> {
        let (client_id, client_secret) = get_client_tokens();
        let token_storage = TwitchTokenStorage {
            user_access_token: self.user_access_token.clone(),
        };
        let credentials = RefreshingLoginCredentials::init(client_id, client_secret, token_storage);
        let config = ClientConfig::new_simple(credentials);

        let (mut incoming_messages, client) = MuniBotTwitchIRCClient::new(config);

        // clone the channel to a new variable before `self` is moved
        let channel = self.channel.to_owned();

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

        // join a channel. this function only returns an error if the passed channel login name is
        // malformed, so in this simple case where the channel name is hardcoded we can ignore the
        // potential error with `unwrap`.
        client.join(channel.to_string()).unwrap();

        client
            .say(channel.to_string(), "i'm here!".to_owned())
            .await
            .unwrap();

        join_handle
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
