use std::env;

use async_trait::async_trait;
use chrono::{DateTime, Local};
use dotenvy::dotenv;
use surrealdb::{
    engine::remote::ws::{self, Ws},
    opt::auth::Database,
    Surreal,
};
use twitch_irc::message::ServerMessage;

use crate::{
    twitch::{
        bot::MuniBotTwitchIRCClient,
        handler::{TwitchHandlerError, TwitchMessageHandler},
    },
    MuniBotError,
};

use serde::{Deserialize, Serialize};

const QUOTE_TABLE: &str = "quote";

/// A quote.
#[derive(Deserialize, Serialize)]
pub struct Quote {
    pub created_at: DateTime<Local>,
    pub quote: String,
    pub invoker: String,
    pub stream_category: String,
    pub stream_title: String,
}

/// A handler for the `!quote` command.
pub struct QuotesHandler {
    db: Surreal<ws::Client>,
}

impl QuotesHandler {
    /// Create a new QuotesHandler, connecting to the database.
    pub async fn new() -> Result<Self, MuniBotError> {
        dotenv().ok(); // TODO: map to MuniBotError::DotenvError

        let database_url = env::var("DATABASE_URL").expect("expected DATABASE_URL to be set"); // TODO: map to MuniBotError::MissingEnv
        let db = Surreal::new::<Ws>(&database_url).await?;
        db.signin(Database {
            namespace: "muni_bot",
            database: "muni_bot",
            username: &env::var("DATABASE_USER").expect("expected DATABASE_USER to be set"),
            password: &env::var("DATABASE_PASS").expect("expected DATABASE_PASS to be set"),
        })
        .await?;

        Ok(Self {
            db: Surreal::new::<Ws>(&database_url).await?,
        })
    }

    /// Add a new quote to the database, returning the new count of quotes
    pub async fn add_new_quote<'a>(
        &mut self,
        new_quote: &Quote,
    ) -> Result<u32, TwitchHandlerError> {
        self.db
            .create::<Vec<Quote>>("new_quote")
            .content(new_quote)
            .await?;

        let count = self
            .db
            .query(format!("SELECT VALUE count() AS count FROM {QUOTE_TABLE};"))
            .await?
            .take::<Option<u32>>(0)?
            .unwrap_or(0);

        Ok(count)
    }

    /// Recall a quote from the database and send it in chat.
    pub async fn recall_quote(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        recipient_channel: &str,
        n_requested: Option<i32>,
    ) -> Result<(), TwitchHandlerError> {
        if let Some(n) = n_requested {
            // recall specific quote
            let mut response = self
                .db
                .query(format!(
                    "SELECT id from {QUOTE_TABLE}
                     ORDER BY created_at
                     LIMIT 1
                     START $n;",
                ))
                .bind(("n", n))
                .await?;

            if let Some(quote) = response.take::<Option<Quote>>(0)? {
                self.send_twitch_message(
                    client,
                    recipient_channel,
                    &format!(r#"here's quote #{}: "{}""#, n, quote.quote),
                )
                .await
            } else {
                self.send_twitch_message(
                    client,
                    recipient_channel,
                    &format!("quote #{n} not found :("),
                )
                .await
            }
        } else {
            // recall random quote
            let mut response = self
                .db
                .query(format!(
                    "SELECT id from {QUOTE_TABLE}
                     ORDER BY rand()
                     LIMIT 1;",
                ))
                .await?;

            if let Some(quote) = response.take::<Option<Quote>>(0)? {
                self.send_twitch_message(
                    client,
                    recipient_channel,
                    &format!(r#"random quote: "{}""#, quote.quote),
                )
                .await
            } else {
                self.send_twitch_message(client, recipient_channel, "no quotes found :(")
                    .await
            }
        }
    }
}

#[async_trait]
impl TwitchMessageHandler for QuotesHandler {
    async fn handle_twitch_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: &ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message && let Some(content) = m.message_text.strip_prefix("!quote").map(str::trim) {
            if content.is_empty() {
                // recall a random quote
                self.recall_quote(client, &m.channel_login, None).await?;
            } else if let Ok(n) = content.parse::<i32>() {
                self.recall_quote(client, &m.channel_login, Some(n)).await?;
            } else {
                    let new_quote = Quote {
                        created_at: Local::now(),
                        quote: content.to_string(),
                        invoker: m.sender.login.to_string(),
                        stream_category: "".to_string(), // TODO
                        stream_title: "".to_string(), // TODO
                    };

                    let quote_count = self.add_new_quote(&new_quote).await?;
                    self.send_twitch_message(client, &m.channel_login, &format!("quote #{quote_count} is in! recorded in the muni history books forever")).await?;
            }
            true
        } else {
            false
        };

        Ok(handled)
    }
}