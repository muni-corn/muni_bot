pub mod commands;
pub mod handler;

use std::{env, sync::Arc};

use dotenvy::dotenv;
use poise::{
    samples::register_globally, serenity_prelude as serenity, Event, Prefix, PrefixFrameworkOptions,
};
use tokio::sync::Mutex;

use crate::{handlers::DiscordHandlerCollection, MuniBotError};

use self::commands::DiscordCommandProvider;

pub struct DiscordState {
    handlers: DiscordHandlerCollection,
}
pub type MutableDiscordState = Arc<Mutex<DiscordState>>;
pub type DiscordContext<'a> = poise::Context<'a, MutableDiscordState, MuniBotError>;
pub type DiscordCommand = poise::Command<MutableDiscordState, MuniBotError>;

pub async fn start_discord_integration(
    handlers: DiscordHandlerCollection,
    command_providers: Vec<Box<dyn DiscordCommandProvider>>,
) {
    dotenv().ok();

    let commands = command_providers
        .iter()
        .flat_map(|provider| provider.commands())
        .collect();

    let options = poise::FrameworkOptions::<MutableDiscordState, MuniBotError> {
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(event_handler(_ctx, event, _framework, _data))
        },
        commands,
        prefix_options: PrefixFrameworkOptions {
            prefix: Some("~".to_string()),
            additional_prefixes: vec![Prefix::Literal("!")],
            ..Default::default()
        },
        ..Default::default()
    };

    poise::Framework::<MutableDiscordState, MuniBotError>::builder()
        .token(
            env::var("DISCORD_TOKEN")
                .expect("no token provided for discord! i can't run without it :("),
        )
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(on_ready(_ctx, _ready, _framework, handlers))
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();
}

async fn on_ready(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<MutableDiscordState, MuniBotError>,
    handlers: DiscordHandlerCollection,
) -> Result<MutableDiscordState, MuniBotError> {
    println!("Logged in as {}", ready.user.name);

    register_globally(ctx, &framework.options().commands)
        .await
        .expect("failed to register commands in guild");

    ctx.set_activity(serenity::Activity::watching("you sleep uwu"))
        .await;

    Ok(Arc::new(Mutex::new(DiscordState { handlers })))
}

async fn event_handler(
    context: &serenity::Context,
    event: &Event<'_>,
    _framework: poise::FrameworkContext<'_, MutableDiscordState, MuniBotError>,
    data: &MutableDiscordState,
) -> Result<(), MuniBotError> {
    if let Event::Message { new_message } = event {
        for handler in data.lock().await.handlers.iter() {
            let mut locked_handler = handler.lock().await;
            let handled_future = locked_handler.handle_discord_message(context, new_message);
            match handled_future.await {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => println!(
                    "discord integration ran into an error executing handlers: {}",
                    e
                ),
            }
        }
    }
    Ok(())
}
