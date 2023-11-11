use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use poise::{serenity_prelude::UserId, Context};

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        DiscordState,
    },
    MuniBotError,
};

pub struct EightBallProvider;

impl EightBallProvider {
    fn get_response(user_id: UserId, question: &str) -> &'static str {
        // determine a response based off of a 5-minute interval, question, and user id
        let to_hash = format!(
            "{}{}{}",
            user_id,
            question,
            chrono::Utc::now().timestamp() / 300
        );

        // hash the value
        let mut hash_state = DefaultHasher::new();
        to_hash.hash(&mut hash_state);
        let hashed = hash_state.finish();

        // use the hash to determine and return the response
        let i = hashed % EIGHT_BALL_RESPONSES.len() as u64;
        EIGHT_BALL_RESPONSES[i as usize]
    }
}

#[poise::command(prefix_command, track_edits, slash_command)]
async fn eight_ball(
    ctx: Context<'_, DiscordState, MuniBotError>,
    question: String,
) -> Result<(), MuniBotError> {
    ctx.say(EightBallProvider::get_response(ctx.author().id, &question))
        .await
        .map_err(|e| DiscordCommandError {
            message: format!("couldn't send message: {}", e),
            command_identifier: "magical".to_string(),
        })?;
    Ok(())
}

const EIGHT_BALL_RESPONSES: [&str; 18] = [
    "yes!",
    "yeah!",
    "sure!",
    "most likely!",
    "if you're hoping so, then sure :>",
    "no!",
    "no...",
    "nope!",
    "not likely.",
    "...maybe we could talk about this a different time.",
    "maybe?",
    "maybe not",
    "probably?",
    "probably not?",
    "eh... i'm not sure.",
    "i can't answer that right now.",
    "could you try rephrasing?",
    "that's up to you!",
];

impl DiscordCommandProvider for EightBallProvider {
    fn commands(&self) -> Vec<poise::Command<DiscordState, crate::MuniBotError>> {
        vec![eight_ball()]
    }
}
