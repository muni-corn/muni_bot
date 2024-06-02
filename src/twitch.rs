use std::{borrow::Cow, str::FromStr};

use twitch_oauth2::Scope;
use url::Url;

pub mod agent;
pub mod bot;
pub mod handler;
pub mod tokens;

pub(crate) const REDIRECT_URI: &str = "http://localhost:6864/twitch";

const SCOPE: [Scope; 8] = [
    Scope::ChannelReadRedemptions,
    Scope::ChannelReadSubscriptions,
    Scope::ModeratorManageAnnouncements,
    Scope::ModeratorManageBannedUsers,
    Scope::Other(Cow::Borrowed("moderator:read:chatters")),
    Scope::ModeratorManageChatMessages,
    Scope::ChatEdit,
    Scope::ChatRead,
];

pub fn get_basic_auth_url() -> Url {
    let mut url = Url::from_str("https://id.twitch.tv/oauth2/authorize").unwrap();
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("env var TWITCH_CLIENT_ID is not set");

    let auth = vec![
        ("response_type", "token"),
        ("client_id", client_id.as_str()),
        ("redirect_uri", REDIRECT_URI),
    ];

    url.query_pairs_mut().extend_pairs(auth);

    url.query_pairs_mut()
        .append_pair("scope", &SCOPE.as_slice().join(" "));

    url
}
