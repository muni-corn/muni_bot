use poise::serenity_prelude::ChannelId;

use super::{DiscordContext, DiscordCommandProvider, DiscordCommand};
use crate::{db::DbItem, handlers::logging::LoggingChannel, MuniBotError};

pub struct AdminCommandProvider;

impl DiscordCommandProvider for AdminCommandProvider {
    fn commands(&self) -> Vec<DiscordCommand> {
        vec![admin()]
    }
}

#[poise::command(
    slash_command,
    hide_in_help,
    required_permissions = "MANAGE_GUILD",
    subcommand_required,
    subcommands("set_log_channel", "stop_logging"),
    ephemeral
)]
async fn admin(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    let mut msg =
        String::from("hi :3 this command has subcommands for managing my server administration tools.");
    if ctx.guild_id().is_some() {
        msg.push_str(" perhaps you meant to use one of them?");
    } else {
        msg.push_str(" you can only use it in a server, silly.");
    }
    ctx.say(msg).await?;
    Ok(())
}

/// set the channel to which log server events will be sent.
#[poise::command(
    rename = "set-log-channel",
    slash_command,
    hide_in_help,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
async fn set_log_channel(ctx: DiscordContext<'_>, channel: ChannelId) -> Result<(), MuniBotError> {
    let db = &ctx.data().db;

    if let Some(guild_id) = ctx.guild_id() {
        LoggingChannel::new(guild_id, channel)
            .update_in_db(db)
            .await?;
        ctx.say(format!("done! log messages will be sent to <#{}>", channel))
            .await?;
    } else {
        ctx.say("this command can only be used in a server, silly.")
            .await?;
    }

    Ok(())
}

/// stop logging messages in the server. re-enable with `set-log-channel`.
#[poise::command(
    rename = "stop-logging",
    slash_command,
    hide_in_help,
    required_permissions = "MANAGE_GUILD"
)]
async fn stop_logging(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    let db = &ctx.data().db;

    if let Some(guild_id) = ctx.guild_id() {
        if let Some(logging_entry) = LoggingChannel::get_from_db(db, guild_id).await? {
            logging_entry.delete_from_db(db).await?;
            ctx.say("done! logging has been disabled for this server.")
                .await?;
        } else {
            ctx.say("no logging channel is set for this server! nothing was done.")
                .await?;
        }
    } else {
        ctx.say("this command can only be used in a server, silly.")
            .await?;
    }

    Ok(())
}
