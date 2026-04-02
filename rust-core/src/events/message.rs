use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::warn;

pub async fn on_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // 1. Spam check
    let spam_result = crate::protection::spam::check(
        data.redis.clone(),
        guild_id,
        msg.author.id,
        &msg.content,
    )
    .await?;

    if spam_result.is_spam {
        handle_spam(ctx, msg, data, &spam_result).await?;
        return Ok(());
    }

    // 2. Link filter
    if crate::protection::links::has_unauthorized_link(&msg.content) {
        msg.delete(ctx).await?;
        msg.channel_id
            .send_message(
                ctx,
                serenity::CreateMessage::new()
                    .content(format!(
                        "⚠️ <@{}> Unauthorized link removed.",
                        msg.author.id
                    ))
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::new().users(vec![msg.author.id]),
                    ),
            )
            .await?;
        return Ok(());
    }

    // 3. AI toxicity check (async, non-blocking)
    let ai_url = data.ai_service_url.clone();
    let http = data.http_client.clone();
    let content = msg.content.clone();
    let author_id = msg.author.id;
    let channel_id = msg.channel_id;
    let db = data.db.clone();
    let ctx_clone = ctx.clone();

    tokio::spawn(async move {
        if let Ok(result) =
            crate::protection::ai::check_toxicity(&http, &ai_url, &content).await
        {
            if result.is_toxic {
                handle_toxic_message(
                    &ctx_clone,
                    db,
                    guild_id,
                    author_id,
                    channel_id,
                    result.score,
                    result.reason,
                )
                .await
                .ok();
            }
        }
    });

    Ok(())
}

async fn handle_spam(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    _data: &Data,
    result: &crate::protection::spam::SpamResult,
) -> Result<(), Error> {
    let guild_id = msg.guild_id.unwrap();
    warn!("Spam detected: {} (violation #{})", msg.author.name, result.violation_count);
    msg.delete(ctx).await?;

    let timeout_secs: i64 = match result.violation_count {
        1 => 60,
        2 => 600,
        3 => 3600,
        _ => 86400,
    };

    let until_str =
        (chrono::Utc::now() + chrono::Duration::seconds(timeout_secs)).to_rfc3339();

    guild_id
        .edit_member(
            ctx,
            msg.author.id,
            serenity::EditMember::new().disable_communication_until(until_str),
        )
        .await?;

    msg.channel_id
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .content(format!(
                    "🚫 <@{}> Spam detected! Muted for {} minute(s). (Violation #{})",
                    msg.author.id,
                    timeout_secs / 60,
                    result.violation_count
                ))
                .allowed_mentions(
                    serenity::CreateAllowedMentions::new().users(vec![msg.author.id]),
                ),
        )
        .await?;

    Ok(())
}

async fn handle_toxic_message(
    ctx: &serenity::Context,
    db: std::sync::Arc<sqlx::PgPool>,
    guild_id: serenity::GuildId,
    author_id: serenity::UserId,
    channel_id: serenity::ChannelId,
    score: f32,
    reason: String,
) -> Result<(), Error> {
    warn!("Toxic content: score={:.2} reason={}", score, reason);

    sqlx::query(
        "INSERT INTO warnings (user_id, guild_id, reason, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(author_id.get() as i64)
    .bind(guild_id.get() as i64)
    .bind(&reason)
    .execute(db.as_ref())
    .await?;

    let warn_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM warnings WHERE user_id = $1 AND guild_id = $2")
            .bind(author_id.get() as i64)
            .bind(guild_id.get() as i64)
            .fetch_one(db.as_ref())
            .await?;

    if warn_count >= 3 {
        let until_str =
            (chrono::Utc::now() + chrono::Duration::minutes(30)).to_rfc3339();

        guild_id
            .edit_member(
                ctx,
                author_id,
                serenity::EditMember::new().disable_communication_until(until_str),
            )
            .await?;

        channel_id
            .send_message(
                ctx,
                serenity::CreateMessage::new()
                    .content(format!(
                        "🔇 <@{}> 3 warnings reached. Muted for 30 minutes.",
                        author_id
                    ))
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::new().users(vec![author_id]),
                    ),
            )
            .await?;
    } else {
        channel_id
            .send_message(
                ctx,
                serenity::CreateMessage::new()
                    .content(format!(
                        "⚠️ <@{}> Warning {}/3: Inappropriate content detected. ({})",
                        author_id, warn_count, reason
                    ))
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::new().users(vec![author_id]),
                    ),
            )
            .await?;
    }

    Ok(())
}
