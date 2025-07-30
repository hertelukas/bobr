mod commands;
use poise::serenity_prelude::{self as serenity, EventHandler, async_trait};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

// User data, which is stored and accessible in all command invocations
struct Data {
    pool: SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("could not connect to database");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await;

    client.unwrap().start().await.unwrap();
}
