use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Bot information
#[poise::command(slash_command)]
pub async fn botinfo(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("🤖 Bot Info")
        .field("Version", "0.1.0", true)
        .field("Language", "Rust + Python", true)
        .field("Library", "Serenity + Poise", true)
        .field(
            "Features",
            "🛡️ Spam Protection\n🚨 Raid Protection\n🔗 Link Filter\n\
             🤖 AI Moderation\n🎭 Auto Role\n✅ Verification\n📋 Logging",
            false,
        )
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Server statistics
#[poise::command(slash_command)]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = guild_id.to_partial_guild(ctx).await?;

    let warn_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM warnings WHERE guild_id = $1")
            .bind(guild_id.get() as i64)
            .fetch_one(ctx.data().db.as_ref())
            .await
            .unwrap_or(0);

    let embed = serenity::CreateEmbed::new()
        .title(format!("📊 {}", guild.name))
        .field("Server ID", guild_id.to_string(), true)
        .field("Owner", format!("<@{}>", guild.owner_id), true)
        .field("Total Warnings", warn_count.to_string(), true)
        .thumbnail(guild.icon_url().unwrap_or_default())
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// View warning history
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn warnings(
    ctx: Context<'_>,
    #[description = "User"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let rows: Vec<(String, chrono::NaiveDateTime)> = sqlx::query_as(
        "SELECT reason, created_at FROM warnings
         WHERE user_id = $1 AND guild_id = $2
         ORDER BY created_at DESC LIMIT 10",
    )
    .bind(user.id.get() as i64)
    .bind(guild_id.get() as i64)
    .fetch_all(ctx.data().db.as_ref())
    .await?;

    if rows.is_empty() {
        ctx.say(format!("✅ {} has no warnings.", user.name))
            .await?;
        return Ok(());
    }

    let list = rows
        .iter()
        .enumerate()
        .map(|(i, (reason, created_at))| {
            format!(
                "**{}**. {} — `{}`",
                i + 1,
                reason,
                created_at.format("%d/%m/%Y")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title(format!("⚠️ {} — Warning History", user.name))
        .description(list)
        .field("Total", rows.len().to_string(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
