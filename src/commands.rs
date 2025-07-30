use prediction_market::LmsrMarket;
use strum::{EnumCount, EnumIter};

use crate::{Context, Error, User};

#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Show help"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn points(ctx: Context<'_>) -> Result<(), Error> {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(ctx.author().id.get() as i64)
        .fetch_optional(&ctx.data().pool)
        .await
        .unwrap();

    if let Some(user) = user {
        ctx.say(format!("You have {:?} points", user.points))
            .await?;
    } else {
        ctx.say("User not yet registered, registering automatically with this message.")
            .await?;
    }

    Ok(())
}

const DEFAULT_LIQUIDITY: f64 = 10.0;

// TODO remove me once public in predictio-market
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, EnumIter)]
pub enum BinaryOutcome {
    Yes,
    No,
}

#[poise::command(slash_command, prefix_command)]
pub async fn new_market(ctx: Context<'_>, title: String, description: String) -> Result<(), Error> {
    let result =
        sqlx::query("INSERT INTO lmsr_markets (liquidity, title, description) VALUES (?, ?, ?)")
            .bind(DEFAULT_LIQUIDITY)
            .bind(title)
            .bind(description)
            .execute(&ctx.data().pool)
            .await
            .unwrap();

    let market_id = result.last_insert_rowid();
    ctx.say(format!("Created market with id: {market_id}"))
        .await?;

    sqlx::query("INSERT INTO shares (market_id, idx, description) VALUES (?, ?, ?)")
        .bind(market_id)
        .bind(0)
        .bind("Yes")
        .execute(&ctx.data().pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO shares (market_id, idx, description) VALUES (?, ?, ?)")
        .bind(market_id)
        .bind(1)
        .bind("No")
        .execute(&ctx.data().pool)
        .await
        .unwrap();

    Ok(())
}
