mod commands;
use poise::serenity_prelude::{self as serenity, EventHandler, Message, async_trait};
use prediction_market::LmsrMarket;
use sqlx::{
    SqlitePool,
    sqlite::{SqlitePoolOptions, SqliteRow},
};
use strum::{EnumCount, IntoEnumIterator};

// User data, which is stored and accessible in all command invocations
struct Data {
    pool: SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler {
    pool: SqlitePool,
}

const INITIAL_POINTS: f64 = 1000.0;

#[async_trait]
impl EventHandler for Handler {
    /// Gives the author one point, or if we are not yet tracking him, inserting
    /// the user with `INITIAL_POINTS` points.
    async fn message(&self, _ctx: poise::serenity_prelude::Context, msg: Message) {
        let user_id = msg.author.id.get() as i64;
        let username = &msg.author.name;

        // Try to get a DB connection (not always necessary, but more efficient in some cases)
        let mut conn = self.pool.acquire().await.unwrap();

        // Check if the user exists
        let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .unwrap();

        if user.is_some() {
            // User exists: increment points by 1
            sqlx::query("UPDATE users SET points = points + 1 WHERE id = ?")
                .bind(user_id)
                .execute(&mut *conn)
                .await
                .unwrap();
        } else {
            // User does not exist: insert with INITIAL_POINTS
            sqlx::query("INSERT INTO users (id, points, username) VALUES (?, ?, ?)")
                .bind(user_id)
                .bind(INITIAL_POINTS)
                .bind(username)
                .execute(&mut *conn)
                .await
                .unwrap();
        }
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct User {
    id: i64,
    points: f64,
    username: String,
}

struct LmsrMarketRow<T: EnumCount + IntoEnumIterator + Copy> {
    market: LmsrMarket<T>,
    title: String,
    description: String,
}

impl<T: EnumCount + IntoEnumIterator + Copy> sqlx::FromRow<'_, SqliteRow> for LmsrMarketRow<T> {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            market: todo!(),
            title: todo!(),
            description: todo!(),
        })
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("could not connect to database");
    let event_pool = pool.clone();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::help(), commands::points(), commands::new_market()],
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
        .event_handler(Handler { pool: event_pool })
        .await;

    client.unwrap().start().await.unwrap();
}
