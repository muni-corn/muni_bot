use async_trait::async_trait;
use twitch_irc::login::{TokenStorage, UserAccessToken};

#[derive(Debug)]
pub struct TwitchTokenStorage {
    pub user_access_token: UserAccessToken,
}

#[async_trait]
impl TokenStorage for TwitchTokenStorage {
    type LoadError = std::io::Error;
    type UpdateError = std::io::Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        Ok(self.user_access_token.clone())
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        token.clone_into(&mut self.user_access_token);
        Ok(())
    }
}
