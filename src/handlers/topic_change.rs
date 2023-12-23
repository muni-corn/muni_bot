use poise::serenity_prelude::{
    interaction::MessageFlags, ButtonStyle, Color, CreateComponents, CreateEmbed,
    InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
};
use std::time::Duration;

use crate::{
    discord::{commands::DiscordCommandProvider, DiscordCommand, DiscordContext},
    MuniBotError,
};

pub struct TopicChangeProvider;

pub const APPROVE_TOPIC_CHANGE: &str = "approve_topic_change";
pub const DENY_TOPIC_CHANGE: &str = "deny_topic_change";

#[poise::command(slash_command, track_edits)]
pub async fn topic_change(ctx: DiscordContext<'_>) -> Result<(), MuniBotError> {
    let reply_handle = ctx
        .send(|m| {
            m.ephemeral(true).embed(|e| {
                e.title("please read this before continuing")
                    .description("if this conversation is making you uncomfortable, this will submit an anonymous request to this channel to change the topic. please only proceed if you genuinely want to change topics and acknowledge that you are not using this feature as a joke. continue?")
                    .color(Color::RED)
            })
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.custom_id(DENY_TOPIC_CHANGE)
                            .label("no, don't request")
                            .style(ButtonStyle::Secondary)
                    })
                    .create_button(|b| {
                        b.custom_id(APPROVE_TOPIC_CHANGE)
                            .label("yes, request topic change")
                            .style(ButtonStyle::Primary)
                    })
                })
            })
        })
        .await?;

    if let Some(interaction) = reply_handle
        .message()
        .await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(60))
        .await
    {
        match interaction.data.custom_id.as_str() {
            APPROVE_TOPIC_CHANGE => {
                interaction
                    .create_interaction_response(ctx.http(), |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| d.content("thanks! i'll send the message. sorry you felt uncomfortable<3 i hope i can help make things more comfy!").flags(MessageFlags::EPHEMERAL | MessageFlags::SUPPRESS_EMBEDS))
                    })
                    .await?;

                ctx.channel_id()
                    .say(
                        &ctx.http(),
                        "this conversation is uncomfortable and a topic change has been requested. let's talk about something else.",
                    )
                    .await?;
            }
            DENY_TOPIC_CHANGE => {
                interaction
                    .create_interaction_response(&ctx.serenity_context().http, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.content("no problem! send in a request any time.")
                                    .flags(MessageFlags::EPHEMERAL | MessageFlags::SUPPRESS_EMBEDS)
                            })
                    })
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
