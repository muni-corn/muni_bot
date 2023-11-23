pub mod commands;
pub mod handler;
pub mod utils;

use std::env;

use dotenvy::dotenv;
use poise::{
    samples::register_globally, serenity_prelude as serenity, Event, Prefix, PrefixFrameworkOptions,
};
use surrealdb::{engine::remote::ws, opt::auth::Database, Surreal};

use crate::{handlers::DiscordHandlerCollection, MuniBotError};

use self::commands::DiscordCommandProvider;

pub struct DiscordState {
    handlers: DiscordHandlerCollection,
    pub db: Surreal<ws::Client>,
}
impl DiscordState {
    pub async fn new(handlers: DiscordHandlerCollection) -> Result<Self, MuniBotError> {
        let database_url = env::var("DATABASE_URL").expect("expected DATABASE_URL to be set"); // TODO: map to MuniBotError::MissingEnv
        let db = Surreal::new::<ws::Ws>(&database_url).await?;
        db.signin(Database {
            namespace: "muni_bot",
            database: "muni_bot",
            username: &env::var("DATABASE_USER").expect("expected DATABASE_USER to be set"),
            password: &env::var("DATABASE_PASS").expect("expected DATABASE_PASS to be set"),
        })
        .await?;

        Ok(Self { handlers, db })
    }
}

pub type DiscordCommand = poise::Command<DiscordState, MuniBotError>;
pub type DiscordContext<'a> = poise::Context<'a, DiscordState, MuniBotError>;
pub type DiscordFrameworkContext<'a> = poise::FrameworkContext<'a, DiscordState, MuniBotError>;

pub async fn start_discord_integration(
    handlers: DiscordHandlerCollection,
    command_providers: Vec<Box<dyn DiscordCommandProvider>>,
) {
    dotenv().ok();

    let commands = command_providers
        .iter()
        .flat_map(|provider| provider.commands())
        .collect();

    let options = poise::FrameworkOptions::<DiscordState, MuniBotError> {
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        commands,
        prefix_options: PrefixFrameworkOptions {
            prefix: Some("~".to_string()),
            additional_prefixes: vec![Prefix::Literal("!")],
            ..Default::default()
        },
        ..Default::default()
    };

    poise::Framework::<DiscordState, MuniBotError>::builder()
        .token(
            env::var("DISCORD_TOKEN")
                .expect("no token provided for discord! i can't run without it :("),
        )
        .setup(move |ctx, ready, framework| Box::pin(on_ready(ctx, ready, framework, handlers)))
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
    framework: &poise::Framework<DiscordState, MuniBotError>,
    handlers: DiscordHandlerCollection,
) -> Result<DiscordState, MuniBotError> {
    println!("Logged in as {}", ready.user.name);

    register_globally(ctx, &framework.options().commands)
        .await
        .expect("failed to register commands in guild");

    ctx.set_activity(serenity::Activity::watching("you sleep uwu"))
        .await;

    DiscordState::new(handlers).await
}

async fn event_handler(
    context: &serenity::Context,
    event: &Event<'_>,
    framework_context: DiscordFrameworkContext<'_>,
    data: &DiscordState,
) -> Result<(), MuniBotError> {
    if let Event::Message { new_message } = event {
        for handler in data.handlers.iter() {
            let mut locked_handler = handler.lock().await;
            let handled_future =
                locked_handler.handle_discord_message(context, framework_context, new_message);
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
