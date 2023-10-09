use std::sync::Arc;

use rocket::{get, State};
use serde::Serialize;
use tokio::sync::Mutex;
use twitch_irc::login::{GetAccessTokenResponse, UserAccessToken};

use crate::{auth_server::REDIRECT_URI, twitch::auth::{state::TwitchAuthState, get_client_tokens}, MuniBotError};

#[get("/?<code>&<state>")]
pub(crate) async fn twitch_oauth_callback(
    code: String,
    state: String,
    auth_state: &State<Arc<Mutex<TwitchAuthState>>>,
) -> Result<String, MuniBotError> {
    if !auth_state.lock().await.csrf_is_valid(&state) {
        return Err(MuniBotError::StateMismatch { got: state });
    }

    let (client_id, client_secret) = get_client_tokens();

    #[derive(Serialize)]
    struct OauthPostBody {
        client_id: String,
        client_secret: String,
        code: String,
        grant_type: String,
        redirect_uri: String,
    }
    let body = OauthPostBody {
        client_id,
        client_secret,
        code,
        grant_type: "authorization_code".to_string(),
        redirect_uri: REDIRECT_URI.to_string(),
    };
    let client = reqwest::Client::new();
    let response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&body)
        .send()
        .await
        .unwrap();

    let get_access_token_response: GetAccessTokenResponse = serde_json::from_str(&response.text().await?)?;
    let user_access_token: UserAccessToken = UserAccessToken::from(get_access_token_response);

    auth_state.auth_tx.send(user_access_token).await?;

    Ok("muni_bot is authorized with twitch! you can close this tab".to_string())
}

#[get("/?<error>&<error_description>", rank = 2)]
pub fn catch_twitch_oauth_error(error: String, error_description: String) -> String {
    eprintln!("caught an error with auth: {error}");
    eprintln!("{error_description}");

    match error.as_str() {
        "access_denied" => String::from("muni_bot was denied access"),
        _ => String::from("muni_bot could not be authorized: {error_description} ({error})"),
    }
}
