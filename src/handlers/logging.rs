use async_trait::async_trait;
use poise::serenity_prelude::{self as serenity, *};
use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Id, Thing},
    Connection, Surreal,
};

use crate::{
    db::DbItem,
    discord::{
        handler::{DiscordEventHandler, DiscordHandlerError},
        DiscordFrameworkContext,
    },
};

pub struct LoggingHandler;

#[async_trait]
impl DiscordEventHandler for LoggingHandler {
    fn name(&self) -> &'static str {
        "logging"
    }

    async fn handle_discord_event(
        &mut self,
        context: &serenity::Context,
        framework: DiscordFrameworkContext<'_>,
        event: &FullEvent,
    ) -> Result<(), DiscordHandlerError> {
        let send = |guild_id: GuildId, embed: CreateEmbed| async move {
            Self::map_result(
                send_message(
                    context,
                    &framework,
                    guild_id,
                    CreateMessage::new().embed(embed),
                )
                .await,
            )
        };

        // fml i don't think there's a better way to do this but here we go :D
        match event {
            FullEvent::AutoModActionExecution { execution } => {
                send(
                    execution.guild_id,
                    simple_embed("automod executed an action", &format!("{:?}", execution)),
                )
                .await
            }

            FullEvent::ChannelCreate { channel } => {
                send(
                    channel.guild_id,
                    simple_embed(
                        "new channel created",
                        &format!("channel {} was created", channel.id.mention()),
                    ),
                )
                .await
            }

            FullEvent::CategoryCreate { category } => {
                let msg = MessageBuilder::new()
                    .push("category ")
                    .push_bold(&category.name)
                    .push(" was created")
                    .build();

                send(
                    category.guild_id,
                    simple_embed("new category created", &msg),
                )
                .await
            }

            FullEvent::CategoryDelete { category } => {
                let msg = MessageBuilder::new()
                    .push("category ")
                    .push_bold(&category.name)
                    .push(" was deleted")
                    .build();

                send(category.guild_id, simple_embed("category deleted", &msg)).await
            }

            FullEvent::ChannelDelete { channel, messages } => {
                let mut msg = MessageBuilder::new();
                msg.push("channel ")
                    .push_bold(&channel.name)
                    .push(" was deleted");

                if let Some(messages) = messages {
                    msg.push("along with its ")
                        .push_bold(format!("{} messages", messages.len()));
                }

                send(
                    channel.guild_id,
                    simple_embed("channel deleted", &msg.build()),
                )
                .await
            }

            FullEvent::GuildBanAddition {
                guild_id,
                banned_user,
            } => {
                let msg = MessageBuilder::new()
                    .push(banned_user.id.mention().to_string())
                    .push(format!(
                        " ({}) was banned",
                        banned_user
                            .global_name
                            .as_ref()
                            .unwrap_or(&banned_user.name),
                    ))
                    .build();

                send(*guild_id, simple_embed("user banned", &msg)).await
            }

            FullEvent::GuildBanRemoval {
                guild_id,
                unbanned_user,
            } => {
                let msg = MessageBuilder::new()
                    .push("on ")
                    .push(unbanned_user.id.mention().to_string())
                    .push(format!(
                        " ({})",
                        unbanned_user
                            .global_name
                            .as_ref()
                            .unwrap_or(&unbanned_user.name),
                    ))
                    .build();

                send(*guild_id, simple_embed("user ban lifted", &msg)).await
            }

            FullEvent::GuildMemberAddition { new_member } => {
                let mut msg = MessageBuilder::new();
                msg.push(new_member.user.id.mention().to_string())
                    .push(" joined!");

                let mut embed = simple_embed("new member joined", &msg.build());
                if let Some(timestamp) = new_member.joined_at {
                    embed = embed.timestamp(timestamp);
                }

                send(new_member.guild_id, embed).await
            }

            FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
                let msg = MessageBuilder::new()
                    .push(user.id.mention().to_string())
                    .push(format!(
                        " ({}) has left",
                        user.global_name.as_ref().unwrap_or(&user.name),
                    ))
                    .build();

                send(*guild_id, simple_embed("member left", &msg)).await
            }

            FullEvent::GuildMemberUpdate { event, .. } => {
                let mut fields = vec![];
                fields.push(("event".to_string(), format!("{:?}", event), false));

                let msg = MessageBuilder::new()
                    .push(event.user.id.mention().to_string())
                    .push(" was updated")
                    .build();

                send(
                    event.guild_id,
                    embed_with_fields("member updated", &msg, fields),
                )
                .await
            }

            FullEvent::GuildRoleCreate { new } => {
                let msg = MessageBuilder::new()
                    .push(new.mention().to_string())
                    .push(" was created")
                    .build();

                send(new.guild_id, simple_embed("new role created", &msg)).await
            }

            FullEvent::GuildRoleDelete {
                guild_id,
                removed_role_id,
                removed_role_data_if_available,
            } => {
                let mut msg = MessageBuilder::new();
                msg.push("role ");

                if let Some(removed_role_data) = removed_role_data_if_available {
                    msg.push_bold(removed_role_data.name.to_string()).push(" ");
                }

                msg.push("with id ")
                    .push_mono(removed_role_id.to_string())
                    .push(" was deleted");

                send(*guild_id, simple_embed("role deleted", &msg.build())).await
            }
            FullEvent::GuildRoleUpdate {
                old_data_if_available,
                new,
            } => {
                let old_rep = old_data_if_available
                    .as_ref()
                    .map_or_else(|| "none".to_string(), |old| format!("{}", old));

                send(
                    new.guild_id,
                    embed_with_fields(
                        "role updated",
                        "",
                        vec![
                            ("old".into(), old_rep, false),
                            ("new".into(), format!("{}", new), false),
                        ],
                    ),
                )
                .await
            }

            FullEvent::GuildUpdate {
                old_data_if_available,
                new_data,
            } => {
                let old_rep = old_data_if_available
                    .as_ref()
                    .map_or_else(|| "none".to_string(), |old| format!("{:?}", old));

                send(
                    new_data.id,
                    embed_with_fields(
                        "guild updated",
                        "",
                        vec![
                            ("old".into(), old_rep, false),
                            ("new".into(), format!("{:?}", new_data), false),
                        ],
                    ),
                )
                .await
            }

            FullEvent::InviteCreate { data } => {
                let title = if data.temporary {
                    "temporary invite created"
                } else {
                    "invite created"
                };

                if let Some(guild_id) = data.guild_id {
                    let mut msg = MessageBuilder::new();
                    if let Some(user) = &data.inviter {
                        msg.push(&format!("by <@{}> ", user.id));
                    }

                    let max_uses = if data.max_uses == 0 {
                        "infinite".to_string()
                    } else {
                        format!("{} maximum", data.max_uses)
                    };

                    msg.push("with code ")
                        .push_mono(&data.code)
                        .push(format!(", for channel <#{}>", data.channel_id))
                        .push(", with ")
                        .push_bold(format!("{} uses", max_uses))
                        .push(", and a lifetime of")
                        .push_bold(format!("{} seconds", data.max_age));

                    send(guild_id, simple_embed(title, &msg.build())).await
                } else {
                    Ok(())
                }
            }

            FullEvent::InviteDelete { data } => {
                if let Some(guild_id) = data.guild_id {
                    let msg = MessageBuilder::new()
                        .push("with code ")
                        .push_mono(&data.code)
                        .push(format!(" for channel <#{}>", data.channel_id))
                        .build();

                    send(guild_id, simple_embed("invite deleted", &msg)).await
                } else {
                    Ok(())
                }
            }

            FullEvent::MessageDelete {
                channel_id,
                deleted_message_id,
                guild_id,
            } => {
                let mut fields = vec![];

                if let Some(cache) = context.cache() {
                    if let Some(deleted_message) = cache.message(channel_id, deleted_message_id) {
                        fields.push((
                            "message content".into(),
                            deleted_message.content_safe(cache),
                            false,
                        ));
                    } else {
                        fields.push((
                            "couldn't retrieve content".into(),
                            "the message wasn't found in the cache.".into(),
                            false,
                        ));
                    }
                } else {
                    fields.push((
                        "couldn't retrieve content".into(),
                        "the cache isn't available".into(),
                        false,
                    ));
                }

                if let Some(guild_id) = guild_id {
                    send(
                        *guild_id,
                        embed_with_fields(
                            "message deleted",
                            &format!(
                                "message {} in <#{}> was deleted",
                                deleted_message_id, channel_id
                            ),
                            fields,
                        ),
                    )
                    .await
                } else {
                    Ok(())
                }
            }

            FullEvent::MessageDeleteBulk {
                channel_id,
                multiple_deleted_messages_ids,
                guild_id,
            } => {
                if let Some(guild_id) = guild_id {
                    send(
                        *guild_id,
                        embed_with_fields(
                            "bulk message deletion executed",
                            &format!(
                                "{} messages were deleted in <#{}>",
                                multiple_deleted_messages_ids.len(),
                                channel_id
                            ),
                            vec![(
                                "deleted message ids".into(),
                                format!("{:?}", multiple_deleted_messages_ids),
                                false,
                            )],
                        ),
                    )
                    .await
                } else {
                    Ok(())
                }
            }

            FullEvent::MessageUpdate {
                old_if_available,
                new,
                event,
            } => {
                dbg!(&old_if_available);
                dbg!(&new);
                dbg!(&event);
                if let Some(guild_id) = event.guild_id {
                    let mut fields = vec![];
                    if let Some(old) = old_if_available {
                        fields.push(("old".into(), old.content.clone(), false));
                    }
                    if let Some(new) = new {
                        fields.push(("new".into(), new.content.clone(), false));
                    }
                    fields.push(("event".into(), format!("{:?}", event), false));
                    send(guild_id, embed_with_fields("message edited", "", fields)).await
                } else {
                    Ok(())
                }
            }

            FullEvent::ReactionRemove { removed_reaction } => {
                if let Some(guild_id) = removed_reaction.guild_id {
                    send(
                        guild_id,
                        simple_embed("reaction removed", &format!("{:?}", removed_reaction)),
                    )
                    .await
                } else {
                    Ok(())
                }
            }

            FullEvent::ReactionRemoveEmoji { removed_reactions } => {
                if let Some(guild_id) = removed_reactions.guild_id {
                    send(
                        guild_id,
                        simple_embed("reaction removed", &format!("{:?}", removed_reactions)),
                    )
                    .await
                } else {
                    Ok(())
                }
            }

            FullEvent::VoiceChannelStatusUpdate {
                old,
                status,
                id,
                guild_id,
            } => {
                let mut fields = vec![];
                if let Some(old) = old {
                    fields.push(("old".to_string(), format!("{:?}", old), false));
                }
                if let Some(status) = status {
                    fields.push(("new".to_string(), format!("{:?}", status), false));
                }

                send(
                    *guild_id,
                    embed_with_fields(
                        "voice channel status updated",
                        &format!("in <#{}>", id),
                        fields,
                    ),
                )
                .await
            }
            _ => Ok(()),
        }
    }
}

impl LoggingHandler {
    const NAME: &'static str = "logging";

    fn map_result<T>(r: anyhow::Result<T>) -> Result<(), DiscordHandlerError> {
        r.map(|_| ())
            .map_err(|e| DiscordHandlerError::from_display(Self::NAME, e))
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggingChannel {
    /// The guild id that owns this logging channel.
    #[serde(skip)]
    guild_id: GuildId,

    /// The actual channel id to which logs will be sent.
    channel_id: ChannelId,
}

const LOGGING_CHANNEL_TABLE: &str = "logging_channel";

#[async_trait]
impl<C: Connection> DbItem<C> for LoggingChannel {
    type GetQuery = GuildId;

    const NAME: &'static str = LOGGING_CHANNEL_TABLE;

    fn get_id(&self) -> Id {
        Id::from(self.guild_id.get())
    }

    async fn get_from_db(
        db: &Surreal<C>,
        guild_id: Self::GetQuery,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let mut result = db
            .query("SELECT * FROM $guild_id;")
            .bind((
                "guild_id",
                Thing {
                    id: Id::from(guild_id.get()),
                    tb: LOGGING_CHANNEL_TABLE.to_string(),
                },
            ))
            .await?;

        Ok(result.take::<Option<Self>>(0)?.map(|mut r| {
            r.guild_id = guild_id;
            r
        }))
    }
}

impl LoggingChannel {
    pub fn new(guild_id: GuildId, channel_id: ChannelId) -> Self {
        Self {
            guild_id,
            channel_id,
        }
    }
}

async fn get_logging_channel_for_guild(
    framework: &DiscordFrameworkContext<'_>,
    guild_id: GuildId,
) -> Result<ChannelId, DiscordHandlerError> {
    let db = &framework.user_data().await.db;
    let logging_channel = LoggingChannel::get_from_db(db, guild_id)
        .await
        .map_err(|e| DiscordHandlerError {
            message: format!("error getting logging channel from db: {e}"),
            handler_name: "logging",
        })?;

    if let Some(logging_channel) = logging_channel {
        Ok(logging_channel.channel_id)
    } else {
        Err(DiscordHandlerError {
            message: "no logging channel found for guild".to_string(),
            handler_name: "logging",
        })
    }
}

async fn send_message(
    context: &serenity::Context,
    framework: &DiscordFrameworkContext<'_>,
    guild_id: GuildId,
    message: CreateMessage,
) -> anyhow::Result<()> {
    let logging_channel = get_logging_channel_for_guild(framework, guild_id).await?;
    logging_channel.send_message(&context.http, message).await?;
    Ok(())
}

fn simple_embed(title: &str, message: &str) -> CreateEmbed {
    CreateEmbed::new().title(title).description(message)
}

fn embed_with_fields(
    title: &str,
    message: &str,
    fields: Vec<(String, String, bool)>,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new().title(title);

    if !message.is_empty() {
        embed = embed.description(message);
    }

    if !fields.is_empty() {
        let fields = fields.into_iter().map(|(name, mut value, inline)| {
            if value.len() > 1024 {
                value = value[..1023].to_string();
                value.push('…');
            }
            (name, value, inline)
        });
        embed = embed.fields(fields);
    }

    embed
}
