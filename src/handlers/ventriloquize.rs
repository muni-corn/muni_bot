use std::{thread, time::Duration};

use poise::{serenity_prelude::CreateMessage, Command, CreateReply};

use crate::{
    discord::{commands::DiscordCommandProvider, DiscordContext, DiscordState},
    MuniBotError,
};

pub struct VentriloquizeProvider;

#[poise::command(slash_command, hide_in_help, check = "is_muni")]
async fn ventriloquize<'a, 'b: 'a>(
    ctx: DiscordContext<'b>,
    message: String,
) -> Result<(), MuniBotError> {
    let channel_id = ctx.channel_id();
    let http = ctx.serenity_context().http.to_owned();

    // notification the command invoker
    let reply = CreateReply::default()
        .ephemeral(true)
        .content("beep boop...");
    ctx.send(reply).await?;

    tokio::spawn(async move {
        // start typing to look like muni_bot is actually typing
        let typing = channel_id.start_typing(&http);

        // wait a minute to simulate typing
        thread::sleep(Duration::from_millis(message.len() as u64 * 25));
        typing.stop();

        // send the message
        let message = CreateMessage::default().content(message);
        if let Err(e) = channel_id.send_message(&http, message).await {
            eprintln!("couldn't send ventriloquization: {e}");
        }
    });

    Ok(())
}

impl DiscordCommandProvider for VentriloquizeProvider {
    fn commands(&self) -> Vec<Command<DiscordState, MuniBotError>> {
        vec![ventriloquize()]
    }
}

async fn is_muni(ctx: DiscordContext<'_>) -> Result<bool, MuniBotError> {
    let ventr_allowlist_str = std::env::var("VENTR_ALLOWLIST").map_err(|e| {
        MuniBotError::Other(format!(
            "couldn't get ventriloquists! (`VENTR_ALLOWLIST` env var): {e}"
        ))
    })?;
    let mut ventr_allowlist = ventr_allowlist_str.split(',');

    Ok(ventr_allowlist.any(|id| id.trim() == ctx.author().id.get().to_string()))
}
