use std::{fs, path::Path};

use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};

use crate::MuniBotError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub db: DbConfig,
    pub discord: DiscordConfig,
    pub twitch: TwitchConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbConfig {
    pub url: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscordConfig {
    #[serde(default)]
    pub invite_link: Option<String>,

    #[serde(default)]
    pub ventriloquists: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TwitchConfig {
    #[serde(default)]
    pub raid_msg_all: Option<String>,

    #[serde(default)]
    pub raid_msg_subs: Option<String>,
}

impl Config {
    /// Reads the config from the file if it exists, otherwise writes the
    /// default config to the file and loads that.
    pub fn read_or_write_default_from<P: AsRef<Path>>(path: P) -> Result<Self, MuniBotError> {
        let p = path.as_ref();

        // check if the path exists
        if !p.exists() {
            // construct default config
            let default = Config::default();

            // format it into a toml string
            let toml_string = toml::to_string_pretty(&default).map_err(|e| {
                MuniBotError::LoadConfig(
                    "couldn't format default config with toml".to_owned(),
                    e.into(),
                )
            })?;

            // write the default config string
            fs::write(p, toml_string).map_err(|e| {
                MuniBotError::LoadConfig(
                    format!(
                        "couldn't write default config to {} (does its parent directory exist?)",
                        p.display()
                    ),
                    e.into(),
                )
            })?;

            // notify we wrote the file
            println!(
                "~~~
  hi! i'm muni_bot! i've written my default configuration file to {} for you :3 <3
~~~",
                p.display()
            );

            // and return the config
            Ok(default)
        } else {
            // read the file to a string
            let raw_string = fs::read_to_string(p).map_err(|e| {
                MuniBotError::LoadConfig(
                    format!("couldn't read contents of {}", p.display()),
                    e.into(),
                )
            })?;

            // parse the string as toml
            let config = toml::from_str(&raw_string).map_err(|e| {
                MuniBotError::LoadConfig(
                    format!("couldn't parse toml from {}", p.display()),
                    e.into(),
                )
            })?;

            // notify we read the config
            println!(
                "~~~
  hiya! configuration has been read from {} ^u^
~~~",
                p.display()
            );

            // return the config
            Ok(config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db: DbConfig {
                url: "127.0.0.1:7654".to_owned(),
                user: "muni_bot".to_owned(),
            },
            discord: DiscordConfig {
                invite_link: None,
                ventriloquists: vec![],
            },
            twitch: TwitchConfig {
                raid_msg_all: None,
                raid_msg_subs: None,
            },
        }
    }
}
