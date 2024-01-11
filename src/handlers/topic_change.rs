use std::time::Duration;

use poise::{
    serenity_prelude::{
        ButtonStyle, Color, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, InteractionResponseFlags,
    },
    CreateReply,
};

use crate::{
    discord::{commands::DiscordCommandProvider, DiscordCommand, DiscordContext},
    MuniBotError,
};

pub struct TopicChangeProvider;

pub const APPROVE_TOPIC_CHANGE: &str = "approve_topic_change";
pub const DENY_TOPIC_CHANGE: &str = "deny_topic_change";

/// request to change the current topic of conversation
#[poise::command(slash_command, track_edits)]
pub async fn topic_change(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    // create the embed, warning message content, and action buttons
    let embed = CreateEmbed::default()
                    .title("please read this before continuing")
                    .description("if this conversation is making you uncomfortable, this will submit an anonymous request to this channel to change the topic. please only proceed if you genuinely want to change topics and acknowledge that you are not using this feature as a joke. continue?")
                    .color(Color::RED);
    let approve_button = CreateButton::new(APPROVE_TOPIC_CHANGE)
        .style(ButtonStyle::Primary)
        .label("yes, request topic change");
    let deny_button = CreateButton::new(DENY_TOPIC_CHANGE)
        .style(ButtonStyle::Secondary)
        .label("no, don't request");
    let action_row = CreateActionRow::Buttons(vec![deny_button, approve_button]);

    // create the warning message
    let reply = CreateReply::default()
        .ephemeral(true)
        .embed(embed)
        .components(vec![action_row]);

    // send initial warning and confirmation message
    let reply_handle = ctx.send(reply).await?;

    // wait for response by button (or timeout after 60 seconds)
    if let Some(interaction) = reply_handle
        .message()
        .await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(60))
        .await
    {
        match interaction.data.custom_id.as_str() {
            // if the user approves the topic change, send a message to the channel and respond with
            // acknowledgement
            APPROVE_TOPIC_CHANGE => {
                let response_message =
                    CreateInteractionResponseMessage::new().flags(InteractionResponseFlags::EPHEMERAL | InteractionResponseFlags::SUPPRESS_EMBEDS).content("thanks! i'll send the message. sorry you felt uncomfortable<3 i hope i can help make things more comfy!");
                let response = CreateInteractionResponse::Message(response_message);
                interaction.create_response(ctx.http(), response).await?;

                ctx.channel_id()
                    .say(
                        &ctx.http(),
                        "this conversation is uncomfortable and a topic change has been requested. let's talk about something else.",
                    )
                    .await?;
            }
            // if the user denies the topic change, only respond with acknowledgement
            DENY_TOPIC_CHANGE => {
                let response_message = CreateInteractionResponseMessage::new()
                    .flags(
                        InteractionResponseFlags::EPHEMERAL
                            | InteractionResponseFlags::SUPPRESS_EMBEDS,
                    )
                    .content("no problem! send in a request any time.");
                let response = CreateInteractionResponse::Message(response_message);
                interaction
                    .create_response(&ctx.serenity_context().http, response)
                    .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

impl DiscordCommandProvider for TopicChangeProvider {
    fn commands(&self) -> Vec<DiscordCommand> {
        vec![topic_change()]
    }
}
