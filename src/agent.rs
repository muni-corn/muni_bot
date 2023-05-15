use serde::Deserialize;
use twitch_irc::login::UserAccessToken;

use crate::MuniBotError;

pub struct Agent {
    client: reqwest::Client,
    user_access_token: UserAccessToken,
}

impl Agent {
    pub fn new(user_access_token: UserAccessToken) -> Self {
        Self {
            client: reqwest::Client::new(),
            user_access_token,
        }
    }

    /// Get the channel info for the given broadcaster ID
    pub async fn get_channel_info(
        &mut self,
        broadcaster_id: &str,
    ) -> Result<ChannelInfo, MuniBotError> {
        self.client
            .get("https://api.twitch.tv/helix/channels")
            .query(&[("broadcaster_id", broadcaster_id)])
            .bearer_auth(&self.user_access_token.access_token)
            .send()
            .await?
            .json()
            .await
            .map_err(MuniBotError::from)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelInfo {
    game_name: String,
    title: String,
}
