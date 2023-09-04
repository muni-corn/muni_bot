#![feature(decl_macro)]
#![feature(let_chains)]

use twitch::bot::MuniBot;
use reqwest::Url;
use rocket::{
    get, http::ContentType, response::Responder, routes, Ignite, Response, Rocket, Shutdown, State,
};
use serde::Serialize;
use std::{
    borrow::Cow,
    error::Error,
    fmt::Display,
    io::Cursor,
    sync::mpsc::{self, Receiver, SendError, SyncSender},
};
use tokio::task::JoinHandle;
use twitch_irc::login::{GetAccessTokenResponse, UserAccessToken};
use twitch_oauth2::{tokens::UserTokenBuilder, Scope};

mod handlers;
mod schema;
mod twitch;

const SCOPE: [Scope; 7] = [
    Scope::ChannelReadRedemptions,
    Scope::ChannelReadSubscriptions,
    Scope::ModeratorManageAnnouncements,
    Scope::Other(Cow::Borrowed("moderator:read:chatters")),
    Scope::ModeratorManageChatMessages,
    Scope::ChatEdit,
    Scope::ChatRead,
];

const REDIRECT_URI: &str = "http://localhost:6864/";

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize token builder
    let client_id = include_str!("./client_id.txt").trim().to_owned();
    let client_secret = include_str!("./client_secret.txt").trim().to_owned();
    let mut token_builder =
        UserTokenBuilder::new(client_id, client_secret, Url::parse(REDIRECT_URI).unwrap())
            .set_scopes(SCOPE.to_vec());

    // get url for auth page
    let (auth_page_url, _) = token_builder.generate_url();

    // initialize rocket and mount routes
    let (auth_tx, auth_rx) = mpsc::sync_channel::<UserAccessToken>(1);
    let rocket = rocket::build()
        .manage(RocketAuthState {
            auth_tx,
            token_builder,
        })
        .mount("/", routes![oauth_code_callback, catch_oauth_error])
        .ignite()
        .await?;

    // get a shutdown handle (to stop rocket after authentication) and launch rocket (no need to
    // await; awaiting will wait for the task to end)
    let shutdown_handle = rocket.shutdown();
    let rocket_handle = launch_rocket(rocket);

    // open web browser to authorize
    let auth_page_handle = open_auth_page(auth_page_url);

    // also start up main handler for main bot logic
    let main_handle = run_bot(auth_rx, shutdown_handle);

    // wait for rocket execution to end
    //
    // one `?` is to check for a `JoinError` and the other is for checking for a rocket launch
    // error
    let _ = rocket_handle.await??;

    // wait for auth page task to end
    auth_page_handle.await?;

    // wait for main task to end
    main_handle.await?;

    Ok(())
}

#[must_use]
fn run_bot(auth_rx: Receiver<UserAccessToken>, shutdown_handle: Shutdown) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        println!("waiting for auth token...");
        let token = auth_rx.recv().unwrap();
        println!("got auth token! shutting down server");

        // we have a token now, so we don't need to listen at our endpoints anymore
        shutdown_handle.notify();
        println!("server is shut down");

        MuniBot::new(token).run().await
    })
}

/// Opens the Twitch autorization page with a new thread. open-rs is not supposed to block, but it
/// does anyways for some reason lol
#[must_use]
fn open_auth_page(auth_page_url: reqwest::Url) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        println!("opening authorization page");
        if let Err(e) = open::that(auth_page_url.to_string()) {
            eprintln!("couldn't open url: {e}");
            eprintln!("to authorize, open up this url: {auth_page_url}");
        } else {
            println!("opened auth page");
        }
    })
}

/// Launches an ignited `Rocket` in a separate thread.
fn launch_rocket(
    rocket: Rocket<rocket::Ignite>,
) -> JoinHandle<Result<Rocket<Ignite>, rocket::Error>> {
    tokio::task::spawn(async { rocket.launch().await })
}

#[get("/?<code>&<state>")]
async fn oauth_code_callback(
    code: String,
    state: String,
    auth_state: &State<RocketAuthState>,
) -> Result<String, MuniBotError> {
    if !auth_state.token_builder.csrf_is_valid(&state) {
        return Err(MuniBotError::StateMismatch { got: state });
    }

    #[derive(Serialize)]
    struct OauthPostBody {
        client_id: String,
        client_secret: String,
        code: String,
        grant_type: String,
        redirect_uri: String,
    }
    let body = OauthPostBody {
        client_id: include_str!("./client_id.txt").trim().to_string(),
        client_secret: include_str!("./client_secret.txt").trim().to_string(),
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

    let user_access_token: UserAccessToken = UserAccessToken::from(serde_json::from_str::<
        GetAccessTokenResponse,
    >(&response.text().await?)?);
    auth_state.auth_tx.send(user_access_token)?;

    Ok("muni_bot is authorized! you can close this tab".to_string())
}

#[get("/?<error>&<error_description>", rank = 2)]
fn catch_oauth_error(error: String, error_description: String) -> String {
    eprintln!("caught an error with auth: {error}");
    eprintln!("{error_description}");

    match error.as_str() {
        "access_denied" => String::from("muni_bot was denied access to your account"),
        _ => String::from("muni_bot could not be authorized: {error_description} ({error})"),
    }
}

#[derive(Debug)]
enum MuniBotError {
    StateMismatch { got: String },
    ParseError(String),
    RequestError(String),
    SendError(String),
}

impl From<serde_json::Error> for MuniBotError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(format!("couldn't parse: {e}"))
    }
}

impl From<reqwest::Error> for MuniBotError {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestError(format!("request failed: {e}"))
    }
}

impl From<SendError<UserAccessToken>> for MuniBotError {
    fn from(e: SendError<UserAccessToken>) -> Self {
        Self::SendError(format!("sending token failed: {e}"))
    }
}

impl Display for MuniBotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuniBotError::StateMismatch { got } => write!(f, "state mismatch from twitch. be careful! this could mean someone is trying to do something malicious. (got code \"{got}\")"),
            MuniBotError::ParseError(e) => write!(f, "parsing failure! {e}"),
            MuniBotError::RequestError(e) => write!(f, "blegh!! {e}"),
            MuniBotError::SendError(e) => write!(f, "send error! {e}"),
        }
    }
}

impl std::error::Error for MuniBotError {}

impl<'req> Responder<'req, 'static> for MuniBotError {
    fn respond_to(self, _request: &'req rocket::Request<'_>) -> rocket::response::Result<'static> {
        let display = format!("{self}");
        Response::build()
            .header(ContentType::Plain)
            .sized_body(display.len(), Cursor::new(display))
            .ok()
    }
}

struct RocketAuthState {
    token_builder: UserTokenBuilder,
    auth_tx: SyncSender<UserAccessToken>,
}
