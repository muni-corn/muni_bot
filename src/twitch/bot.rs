use anyhow::Result;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

use super::{
    agent::TwitchAgent,
    handler::{TwitchHandlerError, TwitchMessageHandler},
};
use crate::{
    config::Config,
    handlers::{
        affection::AffectionHandler, bonk::BonkHandler, greeting::GreetingHandler,
        lift::LiftHandler, lurk::LurkHandler, magical::MagicalHandler, quotes::QuotesHandler,
        raid_msg::RaidMsgHandler, shoutout::ShoutoutHandler, socials::SocialsHandler,
        TwitchHandlerCollection,
    },
};

pub type MuniBotTwitchIRCClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;
pub type MuniBotTwitchIRCError = twitch_irc::Error<SecureTCPTransport, StaticLoginCredentials>;

pub struct TwitchBot {
    message_handlers: TwitchHandlerCollection,
}

impl TwitchBot {
    pub async fn new(config: Config) -> Self {
        Self {
            message_handlers: vec![
                Box::new(QuotesHandler::new(&config.db).await.unwrap()),
                Box::new(BonkHandler),
                Box::new(SocialsHandler),
                Box::new(RaidMsgHandler),
                Box::new(LurkHandler),
                Box::new(GreetingHandler),
                Box::new(LiftHandler::new()),
                Box::new(ShoutoutHandler),
                Box::new(AffectionHandler),
                Box::new(MagicalHandler),
            ],
        }
    }

    pub fn start(
        mut self,
        channel: String,
        token: String,
        bot_config: &Config,
    ) -> Result<JoinHandle<()>> {
        let credentials = StaticLoginCredentials::new("muni__bot".to_owned(), Some(token));
        let cred_config = ClientConfig::new_simple(credentials.clone());
        let agent = TwitchAgent::new(credentials);

        let (mut incoming_messages, client) = MuniBotTwitchIRCClient::new(cred_config);

        // join a channel. this will error if the passed channel login name is
        // malformed.
        if let Err(e) = client.join(channel.clone()) {
            eprintln!("error joining {}'s twitch channel :( {}", channel, e);
        }
        println!("twitch: joined channel {}", channel);

        let bot_config_clone = bot_config.clone();
        let handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let ServerMessage::Notice(notice_msg) = message {
                    println!(
                        "notice received from twitch channel {}: {}",
                        notice_msg.channel_login.unwrap_or("<none>".to_string()),
                        notice_msg.message_text
                    );
                } else if let Err(e) = self
                    .handle_twitch_message(&message, &client, &agent, &bot_config_clone)
                    .await
                {
                    eprintln!("error in message handler! {e}");
                }
            }
        });
        Ok(handle)
    }
}

#[async_trait]
impl TwitchMessageHandler for TwitchBot {
    async fn handle_twitch_message(
        &mut self,
        message: &ServerMessage,
        client: &MuniBotTwitchIRCClient,
        agent: &TwitchAgent<StaticLoginCredentials>,
        config: &Config,
    ) -> Result<bool, TwitchHandlerError> {
        for message_handler in self.message_handlers.iter_mut() {
            // try to handle the message. if the handler determines the message was handled,
            // we'll stop
            if message_handler
                .handle_twitch_message(message, client, agent, config)
                .await?
            {
                return Ok(true);
            }
        }

        return Ok(false);
    }
}
