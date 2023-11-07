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

    /// Get the channel info for the given broadcaster ID
    pub async fn get_channel_info(
        &mut self,
        broadcaster_id: &str,
    ) -> Result<ChannelInfo, MuniBotError> {
        // get token from credentials
        self.creds
            .get_credentials()
            .await
            .map_err(|e| MuniBotError::Other(format!("error getting twitch credentials: {e}")))?
            .token
            // transform to Result so we can return an error if the token is None
            .ok_or_else(|| MuniBotError::Other("no token found".to_string()))
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
            .map_err(MuniBotError::from)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelInfo {
    game_name: String,
    title: String,
}
