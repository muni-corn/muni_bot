use async_trait::async_trait;
use tokio::task::JoinHandle;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

use crate::handlers::{
    bonk::BonkHandler, greeting::GreetingHandler, lurk::LurkHandler, quotes::QuotesHandler,
    raid_msg::RaidMsgHandler, socials::SocialsHandler,
};

use super::{handler::{TwitchHandlerError, TwitchMessageHandler}, agent::TwitchAgent};

pub type MuniBotTwitchIRCClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;
pub type MuniBotTwitchIRCError = twitch_irc::Error<SecureTCPTransport, StaticLoginCredentials>;

pub struct TwitchBot {
    agent: TwitchAgent<StaticLoginCredentials>,
    message_handlers: Vec<Box<dyn TwitchMessageHandler>>,
}

impl TwitchBot {
    pub async fn new(token: String) -> Self {
        let credentials = StaticLoginCredentials::new("muni__bot".to_owned(), Some(token));

        Self {
            agent: TwitchAgent::new(credentials),
            message_handlers: vec![
                Box::new(QuotesHandler::new().await.unwrap()),
                Box::new(BonkHandler),
                Box::new(SocialsHandler),
                Box::new(RaidMsgHandler),
                Box::new(LurkHandler),
                Box::new(GreetingHandler),
            ],
        }
    }

    pub fn start(mut self, channel: String) -> JoinHandle<()> {
        let config = ClientConfig::new_simple(self.agent.get_credentials().clone());

        let (mut incoming_messages, client) = MuniBotTwitchIRCClient::new(config);

        // join a channel. this will panic if the passed channel login name is malformed.
        client.join(channel.clone()).unwrap();

        tokio::spawn(async move {
            client
                .say(channel.to_owned(), "i'm here!".to_owned())
                .await
                .unwrap();

            while let Some(message) = incoming_messages.recv().await {
                if let Err(e) = self.handle_twitch_message(&client, &message).await {
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
