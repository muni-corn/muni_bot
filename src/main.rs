#![feature(decl_macro)]

use reqwest::Url;
use rocket::{get, routes, Ignite, Rocket};
use serde::Serialize;
use std::{borrow::Cow, error::Error};
use tokio::task::JoinHandle;
use twitch_oauth2::{tokens::UserTokenBuilder, Scope};

mod token_storage;

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
    // initialize rocket first and mount routes
    let rocket = rocket::build()
        .mount("/", routes![oauth_code_callback, catch_oauth_error])
        .ignite()
        .await?;

    let client_id = include_str!("./client_id.txt").trim().to_owned();
    let client_secret = include_str!("./client_secret.txt").trim().to_owned();
    let mut token_builder =
        UserTokenBuilder::new(client_id, client_secret, Url::parse(REDIRECT_URI).unwrap())
            .force_verify(true)
            .set_scopes(SCOPE.to_vec());

    let (auth_page_url, _) = token_builder.generate_url();

    // get a shutdown handle (to stop rocket after authentication) and launch rocket (no need to
    // await; awaiting will wait for the task to end)
    let shutdown_handle = rocket.shutdown();
    let rocket_handle = launch_rocket(rocket);

    // open web browser to authorize
    let auth_page_handle = open_auth_page(auth_page_url);

    // wait for rocket execution to end
    //
    // one `?` is to check for a `JoinError` and the other is for checking for a rocket launch
    // error
    let _ = rocket_handle.await??;

    // wait for auth page task to end
    auth_page_handle.await?;

    Ok(())
}

/// Opens the Twitch autorization page with a new thread. open-rs is not supposed to block, but it
/// does anyways for some reason lol
#[must_use]
fn open_auth_page(auth_page_url: reqwest::Url) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        println!("opening authorization page");
        if let Err(e) = open::that(auth_page_url.path()) {
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

#[get("/?<code>&<scope>&<state>")]
async fn oauth_code_callback(code: String, scope: Option<String>, state: Option<String>) -> String {
    println!("authorized! code: {code}");
    if let Some(scope) = scope {
        println!("scope: {scope}");
    }
    if let Some(state) = state {
        println!("state: {state}");
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
    println!(
        "response from token POST: {}",
        response.text().await.unwrap()
    );

    "muni_bot is authorized! you can close this tab".to_string()
}

#[get("/?<error>&<error_description>&<state>", rank = 2)]
fn catch_oauth_error(error: String, error_description: String, state: Option<String>) -> String {
    eprintln!("caught an error with auth: {error}");
    eprintln!("{error_description}");

    match error.as_str() {
        "access_denied" => String::from("muni_bot was denied access to your account"),
        _ => String::from("muni_bot could not be authorized: {error_description} ({error})"),
    }
}
