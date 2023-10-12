use poise::serenity_prelude::MessageBuilder;
use rand::{seq::SliceRandom, Rng};

use crate::{
    discord::{commands::{DiscordCommandProvider, DiscordCommandError}, DiscordState},
    MuniBotError,
};

const PREFIXES: [&'_ str; 5] = ["*giggle!* ", "eee hehe! ", "hehehe! ", "aaa! ", "eep! "];
const CHANCE_OF_PREFIX: f64 = 0.5;
const CHANCE_OF_EXCLAMATION: f64 = 0.5;
const CHANCE_OF_HEART: f64 = 0.25;

pub struct NuzzleProvider;

impl NuzzleProvider {
    pub fn get_response() -> String {
        let mut rng = rand::thread_rng();
        let mut msg = MessageBuilder::new();

        // start by choosing an action
        let action = ACTIONS.choose(&mut rng).unwrap();

        // start the message with a prefix, if decided
        if rng.gen_bool(CHANCE_OF_PREFIX) {
            msg.push(PREFIXES.choose(&mut rng).unwrap());
        }

        // generate optional exclamation mark and heart
        let exclamation = if rng.gen_bool(CHANCE_OF_EXCLAMATION) {
            "!"
        } else {
            ""
        };
        let heart = if rng.gen_bool(CHANCE_OF_HEART) {
            "<3"
        } else {
            ""
        };

        // then push the nuzzle action and build the message
        msg.push_italic(format!("{action}{exclamation}{heart}")).build()
    }
}

/// Nuzzle the good bot!
#[poise::command(slash_command, prefix_command)]
async fn nuzzle(ctx: poise::Context<'_, DiscordState, MuniBotError>) -> Result<(), MuniBotError> {
    ctx.say(NuzzleProvider::get_response())
        .await
        .map_err(|e| DiscordCommandError {
            message: format!("couldn't send message :( {e}"),
            command_identifier: "nuzzle".to_string(),
        })?;

    Ok(())
}

const ACTIONS: [&str; 5] = [
    "nuzzle",
    "nuzzleeeee",
    "nuzzlenuzzle",
    "nuzzles back",
    "nuzznuzz",
];

impl DiscordCommandProvider for NuzzleProvider {
    fn commands(&self) -> Vec<poise::Command<DiscordState, MuniBotError>> {
        vec![nuzzle()]
    }
}
