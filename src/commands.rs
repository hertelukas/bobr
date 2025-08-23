use core::f64;

use poise::serenity_prelude::{Embed, EmbedField};
use prediction_market::{BinaryOutcome, LmsrMarket, Market};

use crate::{Context, Error, FullLmsrMarket, MarketRow, User, UserOwns, get_market};

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
        ctx.say(format!("You have {:.2} points", user.points * 100.0))
            .await?;
    } else {
        ctx.say("User not yet registered, registering automatically with this message.")
            .await?;
    }

    Ok(())
}

const DEFAULT_LIQUIDITY: f64 = 10.0;

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

#[poise::command(slash_command, prefix_command)]
pub async fn markets(ctx: Context<'_>) -> Result<(), Error> {
    let result: Vec<MarketRow> = sqlx::query_as("SELECT * FROM lmsr_markets")
        .fetch_all(&ctx.data().pool)
        .await
        .unwrap();

    let mut embed = Embed::default();
    embed.title = Some("Current Markets".into());
    embed.fields = result
        .iter()
        .filter(|row| !row.is_resolved)
        .map(|row| {
            EmbedField::new(
                format!("{} ({})", &row.title, row.id),
                &row.description,
                false,
            )
        })
        .collect();

    ctx.send(poise::CreateReply::default().embed(embed.into()))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn market(
    ctx: Context<'_>,
    #[description = "Market ID (search with !markets)"] id: i64,
) -> Result<(), Error> {
    let market: FullLmsrMarket<BinaryOutcome> = match get_market(&ctx.data().pool, id).await {
        Some(m) => m,
        None => {
            ctx.say(format!("Market with id {id} could not be found"))
                .await?;
            return Ok(());
        }
    };

    let mut embed = Embed::default();
    embed.title = Some(market.title);
    embed.description = Some(market.description);

    let yes_field = EmbedField::new(
        "Yes",
        format!(
            "Current price: {:.2}",
            market
                .market
                .price(BinaryOutcome::Yes)
                .unwrap_or(f64::INFINITY)
                * 100.0
        ),
        false,
    );

    let no_field = EmbedField::new(
        "No",
        format!(
            "Current price: {:.2}",
            market
                .market
                .price(BinaryOutcome::No)
                .unwrap_or(f64::INFINITY)
                * 100.0
        ),
        false,
    );
    embed.fields = vec![yes_field, no_field];

    ctx.send(poise::CreateReply::default().embed(embed.into()))
        .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn buy(
    ctx: Context<'_>,
    #[description = "Market ID (search with !markets)"] id: i64,
    #[description = "Yes or No option to buy"] option: bool,
    #[description = "Amount of shares"] amount: u64,
) -> Result<(), Error> {
    let user: User = match sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(ctx.author().id.get() as i64)
        .fetch_optional(&ctx.data().pool)
        .await
        .unwrap()
    {
        Some(u) => u,
        None => {
            ctx.say("User not yet registered, registering automatically with this message.")
                .await?;
            return Ok(());
        }
    };

    let mut market: FullLmsrMarket<BinaryOutcome> = match get_market(&ctx.data().pool, id).await {
        Some(m) => m,
        None => {
            ctx.say(format!("Market with id {id} could not be found"))
                .await?;
            return Ok(());
        }
    };

    let outcome = if option {
        BinaryOutcome::Yes
    } else {
        BinaryOutcome::No
    };

    let price = match market.market.buy(outcome, amount) {
        Ok(price) => price,
        Err(e) => {
            ctx.say(format!("Could not buy shares: {e:?} ")).await?;
            return Ok(());
        }
    };

    // This is okay, we just don't save to the database
    if price > user.points {
        ctx.say(format!("Cannot afford {:.2}", price * 100.0))
            .await?;
        return Ok(());
    };

    let dto = market.market.serialize();

    sqlx::query("UPDATE lmsr_markets SET market_volume = ? WHERE id = ?")
        .bind(dto.market_volume)
        .bind(id)
        .execute(&ctx.data().pool)
        .await
        .unwrap();

    let idx: usize = LmsrMarket::<BinaryOutcome>::outcome_index(outcome);

    sqlx::query("UPDATE shares SET amount = ? WHERE market_id = ? AND idx = ?")
        .bind(*dto.shares.get(idx).unwrap() as i64)
        .bind(id)
        .bind(idx as i64)
        .execute(&ctx.data().pool)
        .await
        .unwrap();

    sqlx::query("UPDATE users SET points = ? WHERE id = ?")
        .bind(user.points - price)
        .bind(user.id)
        .execute(&ctx.data().pool)
        .await
        .unwrap();

    ctx.say(format!("Bougt {amount} shares for {:.2}", price * 100.0))
        .await?;

    let user_owns: Option<UserOwns> = sqlx::query_as(
        "SELECT * FROM user_owns WHERE user_id = ? AND market_id = ? AND share_idx = ?",
    )
    .bind(user.id)
    .bind(id)
    .bind(idx as i64)
    .fetch_optional(&ctx.data().pool)
    .await
    .unwrap();

    if user_owns.is_some() {
        sqlx::query("UPDATE user_owns SET amount = amount + ? WHERE user_id = ? AND market_id = ? AND share_idx = ?")
            .bind(amount as i64)
            .bind(user.id)
            .bind(id)
            .bind(idx as i64)
            .execute(&ctx.data().pool)
            .await.unwrap();
    } else {
        sqlx::query(
            "INSERT INTO user_owns (user_id, market_id, share_idx, amount) VALUES (?, ?, ?, ?)",
        )
        .bind(user.id)
        .bind(id)
        .bind(idx as i64)
        .bind(amount as i64)
        .execute(&ctx.data().pool)
        .await
        .unwrap();
    }

    Ok(())
}
