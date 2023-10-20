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
const BOOP_PREFIXES: [&str; 4] = ["ACK! ", "ack! ", "eep! ", "meep! "];
const BOOP_ACTIONS: [&str; 2] = ["boops back!", "bzzzt! @~@"];
const BOOP_ERROR_MESSAGE: &str =
    "thread 'boop handler' panicked at 'your boop has broken the bot!!', src/handlers/bot_affection.rs:60:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace";

const CHANCE_OF_PREFIX: f64 = 0.5;
const CHANCE_OF_EXCLAMATION: f64 = 0.5;
const CHANCE_OF_TILDE: f64 = 0.25;
const CHANCE_OF_HEART: f64 = 0.1;

pub struct BotAffectionProvider;

impl BotAffectionProvider {
    fn get_generic_response(prefixes: &[&str], actions: ActionResponse) -> String {
        let mut rng = rand::thread_rng();
        let mut msg = MessageBuilder::new();

        // start by choosing an action
        let action = actions.pick(&mut rng).unwrap_or("");

        // start the message with a prefix, if decided
        if rng.gen_bool(CHANCE_OF_PREFIX) {
            msg.push(prefixes.choose(&mut rng).unwrap_or(&""));
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
        actions: ActionResponse<'_>,
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

/// Boop the bot!
#[poise::command(slash_command, prefix_command)]
async fn boop(ctx: poise::Context<'_, DiscordState, MuniBotError>) -> Result<(), MuniBotError> {
    if rand::thread_rng().gen_bool(0.01) {
        ctx.say(
            MessageBuilder::new()
                .push_codeblock_safe(BOOP_ERROR_MESSAGE, None)
                .build(),
        )
        .await
        .map_err(|e| DiscordCommandError {
            message: format!("couldn't send message :( {e}"),
            command_identifier: "boop".to_string(),
        })?;

        // sleep for a sec before assuring the user that everything is fine
        std::thread::sleep(std::time::Duration::from_secs(1));

        ctx.say("jk. i'm fine. hehe! :3")
            .await
            .map_err(|e| DiscordCommandError {
                message: format!("couldn't send message :( {e}"),
                command_identifier: "boop".to_string(),
            })
            .map(|_| Ok(()))?
    } else {
        BotAffectionProvider::handle_generic_affection(
            ctx,
            &BOOP_PREFIXES,
            ActionResponse::Rare(&BOOP_ACTIONS, 0.1),
        )
        .await
    }
}

/// Nuzzle the good bot!
#[poise::command(slash_command, prefix_command)]
async fn nuzzle(ctx: poise::Context<'_, DiscordState, MuniBotError>) -> Result<(), MuniBotError> {
    BotAffectionProvider::handle_generic_affection(
        ctx,
        &NUZZLE_PREFIXES,
        ActionResponse::Always(&NUZZLE_ACTIONS),
    )
    .await
}

impl DiscordCommandProvider for BotAffectionProvider {
    fn commands(&self) -> Vec<poise::Command<DiscordState, MuniBotError>> {
        vec![boop(), nuzzle()]
    }
}

enum ActionResponse<'a> {
    /// A collection of actions that always happen.
    Always(&'a [&'a str]),

    /// A collection of actions that may only happen with the probability specified.
    Rare(&'a [&'a str], f64),
}

impl ActionResponse<'_> {
    fn pick(&self, mut rng: impl Rng) -> Option<&str> {
        match self {
            Self::Always(opts) => opts.choose(&mut rng).copied(),
            Self::Rare(opts, p) => {
                if rng.gen_bool(*p) {
                    opts.choose(&mut rng).copied()
                } else {
                    None
                }
            }
        }
    }
}
