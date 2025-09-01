mod commands;
use poise::serenity_prelude::{self as serenity, EventHandler, Message, async_trait};
use prediction_market::{LmsrMarket, LmsrMarketDTO};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use strum::{EnumCount, IntoEnumIterator};

// User data, which is stored and accessible in all command invocations
struct Data {
    pool: SqlitePool,
    http_client: reqwest::Client,
    finnhub_api_key: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Handler {
    pool: SqlitePool,
}

const INITIAL_POINTS: f64 = 10.0;

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
            // User exists: increment points by 0.01
            sqlx::query("UPDATE users SET points = points + 0.01 WHERE id = ?")
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

#[derive(sqlx::FromRow)]
pub(crate) struct UserOwns {
    user_id: i64,
    market_id: i64,
    share_idx: i64,
    amount: i64,
}

#[derive(sqlx::FromRow)]
pub(crate) struct UserStocks {
    user_id: i64,
    stock_symbol: String,
    shares: i64,
    avg_price: f64,
}

pub(crate) struct FullLmsrMarket<T: EnumCount + IntoEnumIterator + Copy + Eq> {
    market: LmsrMarket<T>,
    title: String,
    description: String,
}

#[derive(sqlx::FromRow, Debug)]
struct MarketRow {
    id: i64,
    liquidity: f64,
    is_resolved: bool,
    resolved_idx: Option<i64>,
    market_volume: f64,
    title: String,
    description: String,
}

#[derive(sqlx::FromRow, Debug)]
struct ShareRow {
    market_id: i64,
    idx: i64,
    amount: i64,
    description: String,
}

pub(crate) async fn get_market<T: EnumCount + IntoEnumIterator + Copy + Eq>(
    pool: &SqlitePool,
    market_id: i64,
) -> Option<FullLmsrMarket<T>> {
    let result: MarketRow = sqlx::query_as("SELECT * FROM lmsr_markets WHERE id = ?")
        .bind(market_id)
        .fetch_optional(pool)
        .await
        .unwrap()?;

    let shares: Vec<ShareRow> = sqlx::query_as("SELECT * FROM shares WHERE market_id = ?")
        .bind(market_id)
        .fetch_all(pool)
        .await
        .unwrap();

    let resolved = match result.resolved_idx {
        Some(i) => T::iter().nth(i as usize),
        None => None,
    };

    let market: LmsrMarket<T> = LmsrMarketDTO {
        shares: shares.iter().map(|share| share.amount as u64).collect(),
        liquidity: result.liquidity,
        resolved,
        market_volume: result.market_volume,
    }
    .into();

    Some(FullLmsrMarket {
        market,
        title: result.title,
        description: result.description,
    })
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let connection = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let finnhub_api_key = std::env::var("FINNHUB_API_KEY").expect("missing FINNHUB_API_KEY");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection)
        .await
        .expect("could not connect to database");
    let event_pool = pool.clone();

    let http_client = reqwest::Client::new();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::help(),
                commands::points(),
                commands::new_market(),
                commands::markets(),
                commands::market(),
                commands::buy(),
                commands::sell(),
                commands::portfolio(),
                commands::buy_stock(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { 
                    pool,
                    http_client,
                    finnhub_api_key,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler { pool: event_pool })
        .await;

    client.unwrap().start().await.unwrap();
}
