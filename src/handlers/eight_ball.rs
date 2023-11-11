use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use poise::{
    serenity_prelude::{MessageBuilder, UserId},
    Context,
};

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

        // determine if certain or uncertain responses should be used
        let choices = if ((hashed % 100) as f64) < CHANCE_OF_CERTAINTY * 100.0 {
            &CERTAIN_RESPONSES[..]
        } else {
            &UNCERTAIN_RESPONSES[..]
        };

        // use the hash to determine and return the response
        let i = hashed % choices.len() as u64;
        choices[i as usize]
    }
}

#[poise::command(prefix_command, track_edits, slash_command)]
async fn eight_ball(
    ctx: Context<'_, DiscordState, MuniBotError>,
    question: String,
) -> Result<(), MuniBotError> {
    let eight_ball_response = EightBallProvider::get_response(ctx.author().id, &question);
    let message = MessageBuilder::new()
        .push_quote_line(question)
        .push(eight_ball_response)
        .build();
    ctx.say(message).await.map_err(|e| DiscordCommandError {
        message: format!("couldn't send message: {}", e),
        command_identifier: "magical".to_string(),
    })?;
    Ok(())
}

const CHANCE_OF_CERTAINTY: f64 = 0.7;
const CERTAIN_RESPONSES: [&str; 9] = [
    "yes!",
    "yeah!",
    "sure!",
    "no!",
    "no...",
    "nope!",
    "it is certain uwu",
    "yuh huh!",
    "nuh uh!",
];
const UNCERTAIN_RESPONSES: [&str; 29] = [
    "yeah...?",
    "no...?",
    "maybe?",
    "maybe not",
    "probably?",
    "probably not?",
    "most likely!",
    "not likely.",
    "if you're hoping so, then sure :>",
    "...maybe we could talk about this a different time.",
    "eh... i'm not sure.",
    "i can't answer that right now.",
    "could you try rephrasing?",
    "that's up to you!",
    "i don't know :3",
    "ask muni!",
    "am i qualified to answer that?",
    "i'm just a silly bot, i can't answer that :3",
    "is that what you want?",
    "what do you want the answer to be?",
    "good question! i'll think about it.",
    "mmmm ask again later",
    "wouldn't you like to know~",
    "i think you know the answer to that :3",
    "i'll answer that later~",
    "that's a silly question! cutie x3",
    "*bonk*",
    "eep! could you ask that later?",
    "maybe a dice roll could decide...?",
];

impl DiscordCommandProvider for EightBallProvider {
    fn commands(&self) -> Vec<poise::Command<DiscordState, crate::MuniBotError>> {
        vec![eight_ball()]
    }
}
