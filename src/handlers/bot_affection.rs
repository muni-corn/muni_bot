use poise::serenity_prelude::MessageBuilder;
use rand::{seq::SliceRandom, Rng};

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        DiscordContext, DiscordState, DiscordCommand,
    },
    MuniBotError,
};

const NUZZLE_PREFIXES: [&str; 5] = ["*giggle!*", "eee hehe!", "hehehe!", "aaa!", "eep!"];
const NUZZLE_ACTIONS: [&str; 5] = [
    "nuzzle",
    "nuzzleeeee",
    "nuzzlenuzzle",
    "nuzzles back",
    "nuzznuzz",
];

const BOOP_PREFIXES: [&str; 4] = ["ACK!", "ack!", "eep!", "meep!"];
const BOOP_ACTIONS: [&str; 2] = ["boops back", "@~@ bzzzt"];
const BOOP_ERROR_CHANCE: f64 = 0.01;
const BOOP_ERROR_MESSAGE: &str =
    "thread 'boop handler' panicked at 'your boop has broken the bot!!', src/handlers/bot_affection.rs:60:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace";

const PAT_PREFIXES: [&str; 3] = ["eep!", "hehe!", "meep!"];
const PAT_ACTIONS: [&str; 4] = ["leans into pats", "happy bot noises", "purrs", "is patted"];

const HUG_PREFIXES: [&str; 4] = ["❤❤❤~", "hehe! love ya too~", "hehe~", "huggleeee~"];
const HUG_ACTIONS: [&str; 6] = [
    "hugs back",
    "returns hugs",
    "returns soft hugs",
    "snuggles",
    "huggles",
    "gibs hugs",
];

const KISS_PREFIXES: [&str; 8] = [
    "oh!",
    "meep~!",
    "uwu~",
    "ehehe~",
    "owo~",
    "owo th-thank you~",
    "h-huh??",
    "oh my!",
];
const KISS_ACTIONS: [&str; 5] = [
    "blushes",
    "blushyblush",
    "giggles",
    "hides face",
    "/)///(\\"
];

const CHANCE_OF_EXCLAMATION: f64 = 0.5;
const CHANCE_OF_TILDE: f64 = 0.25;
const CHANCE_OF_HEART: f64 = 0.1;

pub struct BotAffectionProvider;

impl BotAffectionProvider {
    fn get_generic_response(prefixes: ResponseSelection, actions: ResponseSelection) -> String {
        let mut rng = rand::thread_rng();
        let mut msg = MessageBuilder::new();

        // start the message with a prefix, if decided
        if let Some(prefix) = prefixes.pick(&mut rng) {
            msg.push(format!("{prefix} "));
        }

        // start by choosing an action
        if let Some(action) = actions.pick(&mut rng) {
            // generate optional suffixes
            let tilde = get_str_or_empty(&mut rng, "~", CHANCE_OF_TILDE);
            let exclamation = get_str_or_empty(&mut rng, "!", CHANCE_OF_EXCLAMATION);
            let heart = get_str_or_empty(&mut rng, " <3", CHANCE_OF_HEART);

            // then push the nuzzle action and build the message
            msg.push_italic(format!("{action}{tilde}{exclamation}{heart}"));
        }

        let result = msg.build().trim().to_string();
        if result.is_empty() {
            "o///o".to_string()
        } else {
            result
        }
    }

    async fn handle_generic_affection(
        ctx: DiscordContext<'_>,
        prefixes: ResponseSelection<'_>,
        actions: ResponseSelection<'_>,
    ) -> Result<(), MuniBotError> {
        ctx.say(Self::get_generic_response(prefixes, actions))
            .await
            .map_err(|e| DiscordCommandError {
                message: format!("couldn't send message :( {e}"),
                command_identifier: "generic affection".to_string(),
            })?;

        Ok(())
    }
}

/// Returns a string with the given probability, or an empty string.
fn get_str_or_empty(mut rng: impl Rng, s: &str, p: f64) -> &str {
    if rng.gen_bool(p) {
        s
    } else {
        ""
    }
}

/// Boop the bot!
#[poise::command(slash_command, prefix_command)]
async fn boop(
    ctx: poise::Context<'_, DiscordState, MuniBotError>,
) -> Result<(), MuniBotError> {
    // rarely throw a fake error message
    if rand::thread_rng().gen_bool(BOOP_ERROR_CHANCE) {
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
        // otherwise, respond normally
        BotAffectionProvider::handle_generic_affection(
            ctx,
            ResponseSelection::Always(&BOOP_PREFIXES),
            ResponseSelection::Rare(&BOOP_ACTIONS, 0.1),
        )
        .await
    }
}

/// Nuzzle the good bot!
#[poise::command(slash_command, prefix_command)]
async fn nuzzle(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    BotAffectionProvider::handle_generic_affection(
        ctx,
        ResponseSelection::Rare(&NUZZLE_PREFIXES, 0.5),
        ResponseSelection::Always(&NUZZLE_ACTIONS),
    )
    .await
}

/// Smooch the bot ;3
#[poise::command(slash_command, prefix_command)]
async fn kiss(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    // VERY rarely will the boot smooch back.
    if rand::thread_rng().gen_bool(0.00001) {
        ctx.say("smooch~")
            .await
            .map_err(|e| DiscordCommandError {
                message: format!("couldn't send message :( {e}"),
                command_identifier: "kiss".to_string(),
            })
            .map(|_| Ok(()))?
    } else {
        BotAffectionProvider::handle_generic_affection(
            ctx,
            ResponseSelection::Rare(&KISS_PREFIXES, 0.9),
            ResponseSelection::Rare(&KISS_ACTIONS, 0.3),
        )
        .await
    }
}

/// Pat the bot! >w<
#[poise::command(slash_command, prefix_command)]
async fn pat(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    BotAffectionProvider::handle_generic_affection(
        ctx,
        ResponseSelection::Always(&PAT_PREFIXES),
        ResponseSelection::Always(&PAT_ACTIONS),
    )
    .await
}

/// Hug the bot! <3
#[poise::command(slash_command, prefix_command)]
async fn hug(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    BotAffectionProvider::handle_generic_affection(
        ctx,
        ResponseSelection::Always(&HUG_PREFIXES),
        ResponseSelection::Always(&HUG_ACTIONS),
    )
    .await
}

impl DiscordCommandProvider for BotAffectionProvider {
    fn commands(&self) -> Vec<DiscordCommand> {
        vec![boop(), nuzzle(), kiss(), pat(), hug()]
    }
}

enum ResponseSelection<'a> {
    /// A collection of responses that will always have a selection.
    Always(&'a [&'a str]),

    /// A collection of responses that may only happen with the probability specified.
    Rare(&'a [&'a str], f64),
}

impl ResponseSelection<'_> {
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
