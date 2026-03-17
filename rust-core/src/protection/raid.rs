use poise::serenity_prelude as serenity;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

// Raid thresholds
const JOIN_THRESHOLD: i64 = 10;    // 10 saniyede
const TIME_WINDOW: u64 = 10;       // 10 saniye
const MIN_ACCOUNT_AGE_DAYS: i64 = 7; // Accounts younger than 7 days are suspicious

pub async fn check_and_update(
    mut redis: redis::aio::ConnectionManager,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    account_created: serenity::Timestamp,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Join rate tracking
    let join_key = format!("joins:{}", guild_id);
    redis.zadd::<_, _, _, ()>(&join_key, now, format!("{}:{}", user_id, now)).await?;
    redis.zrembyscore::<_, _, _, ()>(&join_key, 0u64, now - TIME_WINDOW).await?;
    redis.expire::<_, ()>(&join_key, 60).await?;

    let recent_joins: i64 = redis.zcard(&join_key).await?;

    // Account age check
    let created: chrono::DateTime<chrono::Utc> = account_created.into();
    let account_age_days = (chrono::Utc::now() - created).num_days();
    let is_new_account = account_age_days < MIN_ACCOUNT_AGE_DAYS;

    // Lockdown check
    let lockdown_key = format!("lockdown:{}", guild_id);
    let lockdown_active: bool = redis.exists(&lockdown_key).await?;

    if lockdown_active {
        return Ok(true);
    }

    // Raid detected: too many joins OR very new accounts
    if recent_joins >= JOIN_THRESHOLD {
        // Start lockdown (5 minutes)
        redis.set_ex::<_, _, ()>(&lockdown_key, "1", 300).await?;
        tracing::warn!(
            "LOCKDOWN INITIATED - Guild: {} | {} joins in the last {}s",
            guild_id, TIME_WINDOW, recent_joins
        );
        return Ok(true);
    }

    // New account + fast join combo (lower threshold)
    if is_new_account && recent_joins >= JOIN_THRESHOLD / 2 {
        return Ok(true);
    }

    Ok(false)
}

pub async fn is_lockdown_active(
    mut redis: redis::aio::ConnectionManager,
    guild_id: serenity::GuildId,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let lockdown_key = format!("lockdown:{}", guild_id);
    Ok(redis.exists(&lockdown_key).await?)
}

pub async fn end_lockdown(
    mut redis: redis::aio::ConnectionManager,
    guild_id: serenity::GuildId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lockdown_key = format!("lockdown:{}", guild_id);
    redis.del::<_, ()>(&lockdown_key).await?;
    Ok(())
}
