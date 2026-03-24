use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Issue a warning to a user
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "User to warn"] user: serenity::User,
    #[description = "Reason for warning"] reason: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO warnings (user_id, guild_id, reason, moderator_id, created_at)
         VALUES ($1, $2, $3, $4, NOW())"
    )
    .bind(user.id.get() as i64)
    .bind(guild_id.get() as i64)
    .bind(&reason)
    .bind(ctx.author().id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    let warn_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM warnings WHERE user_id = $1 AND guild_id = $2"
    )
    .bind(user.id.get() as i64)
    .bind(guild_id.get() as i64)
    .fetch_one(ctx.data().db.as_ref())
    .await?;

    // DM the user
    let dm_embed = serenity::CreateEmbed::new()
        .title("⚠️ You Received a Warning")
        .field("Server", guild_id.to_string(), true)
        .field("Reason", &reason, false)
        .field("Total Warnings", warn_count.to_string(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    user.dm(ctx, serenity::CreateMessage::new().embed(dm_embed)).await.ok();

    let embed = serenity::CreateEmbed::new()
        .title("⚠️ Warning Issued")
        .field("User", format!("{} ({})", user.name, user.id), true)
        .field("Reason", &reason, false)
        .field("Total Warnings", warn_count.to_string(), true)
        .field("Moderator", ctx.author().name.clone(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Mute a user
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "User to mute"] user: serenity::User,
    #[description = "Duration (minutes)"] minutes: u32,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());

    let until_dt = chrono::Utc::now() + chrono::Duration::minutes(minutes as i64);
    let until_str = until_dt.to_rfc3339();
    let until = serenity::Timestamp::parse(&until_str)?;

    guild_id
        .edit_member(ctx, user.id, serenity::EditMember::new().disable_communication_until(until))
        .await?;

    let embed = serenity::CreateEmbed::new()
        .title("🔇 User Timed Out")
        .field("User", format!("{} ({})", user.name, user.id), true)
        .field("Duration", format!("{} minutes", minutes), true)
        .field("Reason", &reason, false)
        .field("Moderator", ctx.author().name.clone(), true)
        .color(serenity::Colour::RED)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Ban a user
#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Reason"] reason: Option<String>,
    #[description = "Delete message days (0-7)"] delete_days: Option<u8>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());
    let delete_days = delete_days.unwrap_or(0).min(7);

    guild_id.ban_with_reason(ctx, user.id, delete_days, &reason).await?;

    let embed = serenity::CreateEmbed::new()
        .title("🔨 User Banned")
        .field("User", format!("{} ({})", user.name, user.id), true)
        .field("Reason", &reason, false)
        .field("Moderator", ctx.author().name.clone(), true)
        .color(serenity::Colour::RED)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Remove a ban
#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "User ID"] user_id: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let id: u64 = user_id.parse()?;
    guild_id.unban(ctx, serenity::UserId::new(id)).await?;
    ctx.say(format!("✅ <@{}> has been unbanned.", id)).await?;
    Ok(())
}

/// Delete messages
#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
pub async fn clear(
    ctx: Context<'_>,
    #[description = "Number of messages to delete (1-100)"] amount: u8,
) -> Result<(), Error> {
    let amount = amount.clamp(1, 100);
    let messages = ctx.channel_id().messages(ctx, serenity::GetMessages::new().limit(amount)).await?;
    let ids: Vec<serenity::MessageId> = messages.iter().map(|m| m.id).collect();
    let count = ids.len();
    ctx.channel_id().delete_messages(ctx, &ids).await?;
    ctx.send(
        poise::CreateReply::default()
            .content(format!("🗑️ Deleted {} message(s).", count))
            .ephemeral(true),
    ).await?;
    Ok(())
}

/// Manual lockdown
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn lockdown(
    ctx: Context<'_>,
    #[description = "on or off"] state: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let mut conn = ctx.data().redis.lock().await;
    let lockdown_key = format!("lockdown:{}", guild_id);

    match state.to_lowercase().as_str() {
        "on" => {
            redis::AsyncCommands::set_ex::<_, _, ()>(&mut *conn, &lockdown_key, "1", 3600usize).await?;
            ctx.say("🔒 **LOCKDOWN ACTIVE!** New member joins are blocked. Use `/lockdown off` to disable.").await?;
        }
        "off" => {
            redis::AsyncCommands::del::<_, ()>(&mut *conn, &lockdown_key).await?;
            ctx.say("🔓 Lockdown lifted. Server is back to normal.").await?;
        }
        _ => {
            ctx.say("❌ Invalid option. Use `on` or `off`.").await?;
        }
    }
    Ok(())
}
