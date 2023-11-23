use async_trait::async_trait;
use tokio::task::JoinHandle;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

use crate::handlers::{
    bonk::BonkHandler, greeting::GreetingHandler, lift::LiftHandler, lurk::LurkHandler,
    quotes::QuotesHandler, raid_msg::RaidMsgHandler, shoutout::ShoutoutHandler,
    socials::SocialsHandler,
};

use super::{
    agent::TwitchAgent,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};

pub type MuniBotTwitchIRCClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;
pub type MuniBotTwitchIRCError = twitch_irc::Error<SecureTCPTransport, StaticLoginCredentials>;

pub struct TwitchBot {
    message_handlers: Vec<Box<dyn TwitchMessageHandler>>,
}

impl TwitchBot {
    pub async fn new() -> Self {
        Self {
            message_handlers: vec![
                Box::new(QuotesHandler::new().await.unwrap()),
                Box::new(BonkHandler),
                Box::new(SocialsHandler),
                Box::new(RaidMsgHandler),
                Box::new(LurkHandler),
                Box::new(GreetingHandler),
                Box::new(LiftHandler::new()),
                Box::new(ShoutoutHandler),
            ],
        }
    }

    pub fn start(mut self, channel: String, token: String) -> JoinHandle<()> {
        let credentials = StaticLoginCredentials::new("muni__bot".to_owned(), Some(token));
        let config = ClientConfig::new_simple(credentials.clone());
        let agent = TwitchAgent::new(credentials);

        let (mut incoming_messages, client) = MuniBotTwitchIRCClient::new(config);

        // join a channel. this will panic if the passed channel login name is malformed.
        client.join(channel.clone()).unwrap();
        println!("joined twitch channel {}", channel);

        tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let Err(e) = self.handle_twitch_message(&message, &client, &agent).await {
                    eprintln!("error in message handler! {e}");
                }
            }
        })
    }
}

#[async_trait]
impl TwitchMessageHandler for TwitchBot {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        agent: &TwitchAgent<StaticLoginCredentials>,
    ) -> Result<bool, TwitchHandlerError> {
        for message_handler in self.message_handlers.iter_mut() {
            // try to handle the message. if the handler determines the message was handled, we'll
            // stop
            if message_handler
                .handle_twitch_message(message, client, agent)
                .await?
            {
                return Ok(true);
            }
        }

        return Ok(false);
    }
}
