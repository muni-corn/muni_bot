use twitch_api::{helix::channels::ChannelInformation, types::UserId, HelixClient};

use super::tokens::TwitchAuth;

pub struct TwitchAgent<'a> {
    helix_client: HelixClient<'a, reqwest::Client>,
    auth: TwitchAuth,
}

impl<'a> TwitchAgent<'a> {
    pub fn new(auth: TwitchAuth) -> Self {
        let helix_client = HelixClient::default();
        Self {
            helix_client,
            auth,
        }
    }

    pub fn get_bot_id(&self) -> &UserId {
        &self.auth.get_user_token().user_id
    }

    pub fn get_helix_client(&self) -> &HelixClient<'a, reqwest::Client> {
        &self.helix_client
    }

    pub fn get_auth(&self) -> &TwitchAuth {
        &self.auth
    }

    /// Get the channel info for the given broadcaster ID
    pub async fn get_channel_info(
        &self,
        broadcaster_id: &str,
    ) -> Result<Option<ChannelInformation>, anyhow::Error> {
        Ok(self
            .helix_client
            .get_channel_from_id(broadcaster_id, self.auth.get_user_token())
            .await?)
    }
}
