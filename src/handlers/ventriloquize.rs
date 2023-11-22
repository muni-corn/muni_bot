use std::{thread, time::Duration};

use poise::{serenity_prelude::MessageBuilder, Command};

use crate::{
    discord::{
        commands::{DiscordCommandError, DiscordCommandProvider},
        DiscordContext, MutableDiscordState,
    },
    MuniBotError,
};

pub struct VentriloquizeProvider;

#[poise::command(slash_command, hide_in_help, check = "is_muni")]
async fn ventriloquize(ctx: DiscordContext<'_>, message: String) -> Result<(), MuniBotError> {
    // start typing to look like muni_bot is actually typing
    let typing_opt = ctx
        .channel_id()
        .start_typing(&ctx.serenity_context().http)
        .ok();

    // wait a minute to simulate typing
    if let Some(typing) = typing_opt {
        thread::sleep(Duration::from_millis(message.len() as u64 * 10));
        typing.stop();
    }

    // send the message
    let send_result = ctx
        .channel_id()
        .send_message(&ctx.serenity_context().http, |m| m.content(message))
        .await;

    // notify the command invoker of the result
    ctx.send(move |f| {
        let notification = match send_result {
            Ok(_) => "done!".to_string(),
            Err(e) => format!("failed to send message: {e}"),
        };
        f.ephemeral(true).content(notification)
    })
    .await
    .map_err(|e| DiscordCommandError {
        message: format!("failed to send ventriloquized message: {e}"),
        command_identifier: "ventriloquize".to_string(),
    })?;

    Ok(())
}

impl DiscordCommandProvider for VentriloquizeProvider {
    fn commands(&self) -> Vec<Command<MutableDiscordState, MuniBotError>> {
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

    Ok(ventr_allowlist.any(|id| id.trim() == ctx.author().id.0.to_string()))
}
