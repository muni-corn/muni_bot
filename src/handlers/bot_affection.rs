use poise::serenity_prelude::MessageBuilder;
use rand::{seq::SliceRandom, Rng};

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        DiscordState,
    },
    MuniBotError,
};

const NUZZLE_PREFIXES: [&str; 5] = ["*giggle!* ", "eee hehe! ", "hehehe! ", "aaa! ", "eep! "];
const NUZZLE_ACTIONS: [&str; 5] = [
    "nuzzle",
    "nuzzleeeee",
    "nuzzlenuzzle",
    "nuzzles back",
    "nuzznuzz",
];

const CHANCE_OF_PREFIX: f64 = 0.5;
const CHANCE_OF_EXCLAMATION: f64 = 0.5;
const CHANCE_OF_TILDE: f64 = 0.25;
const CHANCE_OF_HEART: f64 = 0.1;

pub struct BotAffectionProvider;

impl BotAffectionProvider {
    pub fn get_generic_response(prefixes: &[&str], actions: &[&str]) -> String {
        let mut rng = rand::thread_rng();
        let mut msg = MessageBuilder::new();

        // start by choosing an action
        let action = actions.choose(&mut rng).unwrap();

        // start the message with a prefix, if decided
        if rng.gen_bool(CHANCE_OF_PREFIX) {
            msg.push(prefixes.choose(&mut rng).unwrap());
        }

        // generate optional suffixes
        let tilde = if rng.gen_bool(CHANCE_OF_TILDE) {
            "~"
        } else {
            ""
        };
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
        msg.push_italic(format!("{action}{tilde}{exclamation}{heart}"))
            .build()
    }

    async fn handle_generic_affection(
        ctx: poise::Context<'_, DiscordState, MuniBotError>,
        prefixes: &[&str],
        actions: &[&str],
    ) -> Result<(), MuniBotError> {
        ctx.say(Self::get_generic_response(prefixes, actions))
            .await
            .map_err(|e| DiscordCommandError {
                message: format!("couldn't send message :( {e}"),
                command_identifier: "nuzzle".to_string(),
            })?;

        Ok(())
    }
}

/// Nuzzle the good bot!
#[poise::command(slash_command, prefix_command)]
async fn nuzzle(ctx: poise::Context<'_, DiscordState, MuniBotError>) -> Result<(), MuniBotError> {
    BotAffectionProvider::handle_generic_affection(ctx, &NUZZLE_PREFIXES, &NUZZLE_ACTIONS).await
}

impl DiscordCommandProvider for BotAffectionProvider {
    fn commands(&self) -> Vec<poise::Command<DiscordState, MuniBotError>> {
        vec![nuzzle()]
    }
}
