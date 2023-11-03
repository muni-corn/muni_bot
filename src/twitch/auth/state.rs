use std::borrow::Cow;

use reqwest::Url;
use tokio::sync::mpsc::{Receiver, Sender};
use twitch_irc::login::UserAccessToken;
use twitch_oauth2::{tokens::UserTokenBuilder, Scope};

use crate::auth_server::REDIRECT_URI;

use super::get_client_tokens;

const SCOPE: [Scope; 7] = [
    Scope::ChannelReadRedemptions,
    Scope::ChannelReadSubscriptions,
    Scope::ModeratorManageAnnouncements,
    Scope::Other(Cow::Borrowed("moderator:read:chatters")),
    Scope::ModeratorManageChatMessages,
    Scope::ChatEdit,
    Scope::ChatRead,
];

pub struct TwitchAuthState {
    auth_page_url: Url,
    token_builder: UserTokenBuilder,
    pub auth_tx: Sender<UserAccessToken>,
}

impl TwitchAuthState {
    pub fn new() -> (Self, Receiver<UserAccessToken>) {
        let twitch_redirect_uri = REDIRECT_URI;

        // initialize token builder
        let (client_id, client_secret) = get_client_tokens();
        let mut token_builder = UserTokenBuilder::new(
            client_id,
            client_secret,
            Url::parse(twitch_redirect_uri).unwrap(),
        )
        .set_scopes(SCOPE.to_vec());

        // get url for auth page
        let (auth_page_url, _) = token_builder.generate_url();

        // create channel for receiving access token
        let (auth_tx, auth_rx) = tokio::sync::mpsc::channel(1);

        (
            Self {
                auth_page_url,
                token_builder,
                auth_tx,
            },
            auth_rx,
        )
    }

    pub fn get_auth_page_url(&self) -> &Url {
        &self.auth_page_url
    }

    pub fn csrf_is_valid(&self, state: &str) -> bool {
        self.token_builder.csrf_is_valid(state)
    }
}
