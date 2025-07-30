use crate::{Context, Error, User};

#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("foo").await?;
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
