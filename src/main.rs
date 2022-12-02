#![feature(decl_macro)]

use rocket::{error::LaunchError, get, routes, Rocket};
use std::error::Error;

mod token_storage;

const SCOPE: [&str; 7] = [
    "channel:read:redemptions",
    "channel:read:subscriptions",
    "moderator:manage:announcements",
    "moderator:read:chatters",
    "moderator:manage:chat_messages",
    "chat:edit",
    "chat:read",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rocket_config = rocket::Config::build(rocket::config::Environment::Development)
        .port(8532)
        .finalize()
        .unwrap();
    let rocket =
        rocket::custom(rocket_config).mount("/", routes![oauth_code_callback, catch_oauth_error]);

    // open web browser to authorize
    let client_id = include_str!("./client_id.txt").to_owned();
    let scope_str = SCOPE
        .iter()
        .map(|s| s.replace(':', "%3A"))
        .collect::<Vec<String>>()
        .join("+");

    let auth_page_handle = open_auth_page(client_id, scope_str);

    let rocket_handle = launch_rocket(rocket);

    rocket_handle.join().unwrap();
    auth_page_handle.join().unwrap();

    Ok(())
}

/// Opens the Twitch autorization page with a new thread. open-rs is not supposed to block, but it
/// does anyways for some reason lol
#[must_use]
fn open_auth_page(client_id: String, scope: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        println!("opening authorization page");
        let url = format!("https://id.twitch.tv/oauth2/authorize?response_type=code&client_id={client_id}&redirect_uri=http://localhost:8532/&scope={scope}");
        if let Err(e) = open::that(&url) {
            eprintln!("couldn't open url: {e}");
            eprintln!("to authorize, open up this url: {url}");
        }
        println!("opened auth page");
    })
}

/// Launches a (hopefully already mounted) `Rocket` in a separate thread.
#[must_use]
fn launch_rocket(rocket: Rocket) -> std::thread::JoinHandle<LaunchError> {
    std::thread::spawn(move || rocket.launch())
}

#[get("/?<code>&<scope>&<state>")]
fn oauth_code_callback(code: String, scope: Option<String>, state: Option<String>) -> &'static str {
    println!("authorized! code: {code}");
    if let Some(scope) = scope {
        println!("scope: {scope}");
    }
    if let Some(state) = state {
        println!("state: {state}");
    }

    "muni_bot is authorized! you can close this tab"
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
