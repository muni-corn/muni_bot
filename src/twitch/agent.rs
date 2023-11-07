use std::{
    error::Error,
    fmt::{Debug, Display},
};

use serde::Deserialize;
use twitch_irc::login::LoginCredentials;

use crate::MuniBotError;

pub struct TwitchAgent<C: LoginCredentials> {
    client: reqwest::Client,
    creds: C,
}

impl<C: LoginCredentials> TwitchAgent<C> {
    pub fn new(creds: C) -> Self {
        Self {
            client: reqwest::Client::new(),
            creds,
        }
    }

    /// Get the credentials used by this agent
    pub fn get_credentials(&self) -> &C {
        &self.creds
    }

    /// Get the channel info for the given broadcaster ID
    pub async fn get_channel_info(
        &self,
        broadcaster_id: &str,
    ) -> Result<ChannelInfo, TwitchAgentError> {
        // get token from credentials
        self.creds
            .get_credentials()
            .await
            .map_err(|e| TwitchAgentError::CredentialsError(format!("{e}")))?
            .token
            // transform to Result so we can return an error if the token is None
            .ok_or_else(|| TwitchAgentError::MissingCredentials)
            // map to use the token and create a Future to make the api call
            .map(|t| async move {
                self.client
                    .get("https://api.twitch.tv/helix/channels")
                    .query(&[("broadcaster_id", broadcaster_id)])
                    .bearer_auth(t)
                    .send()
                    .await?
                    .json()
                    .await
            })?
            // await the Future
            .await
            .map_err(TwitchAgentError::from)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelInfo {
    pub game_name: String,
    pub title: String,
}

#[derive(Debug)]
pub enum TwitchAgentError {
    CredentialsError(String),
    MissingCredentials,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for TwitchAgentError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl Display for TwitchAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwitchAgentError::CredentialsError(e) => {
                write!(f, "error with twitch credentials: {e}")
            }
            TwitchAgentError::MissingCredentials => write!(f, "twitch credentials went missing"),
            TwitchAgentError::ReqwestError(e) => write!(f, "twitch agent request error: {e}"),
        }
    }
}

impl Error for TwitchAgentError {}
