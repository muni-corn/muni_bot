use async_trait::async_trait;
use twitch_irc::login::{TokenStorage, UserAccessToken};

#[derive(Debug)]
pub struct MuniBotTokenStorage {
    pub user_access_token: UserAccessToken,
}

#[async_trait]
impl TokenStorage for MuniBotTokenStorage {
    type LoadError = std::io::Error;
    type UpdateError = std::io::Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        Ok(self.user_access_token.clone())
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.user_access_token = token.to_owned();
        Ok(())
    }
}
