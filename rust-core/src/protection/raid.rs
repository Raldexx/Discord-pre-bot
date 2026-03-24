use poise::serenity_prelude as serenity;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Raid thresholds
const JOIN_THRESHOLD: i64 = 10;
const TIME_WINDOW: u64 = 10;
const MIN_ACCOUNT_AGE_DAYS: i64 = 7; // Accounts younger than 7 days are suspicious

pub async fn check_and_update(
    redis: Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    account_created: serenity::Timestamp,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = redis.lock().await;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Join rate tracking
    let join_key = format!("joins:{}", guild_id);
    conn.zadd::<_, _, _, ()>(&join_key, now, format!("{}:{}", user_id, now)).await?;
    conn.zrembyscore::<_, _, _, ()>(&join_key, 0u64, now - TIME_WINDOW).await?;
    conn.expire::<_, ()>(&join_key, 60).await?;
    let recent_joins: i64 = conn.zcard(&join_key).await?;

    // Account age check
    let created_secs = account_created.unix_timestamp();
    let now_secs = chrono::Utc::now().timestamp();
    let account_age_days = (now_secs - created_secs) / 86400;
    let is_new_account = account_age_days < MIN_ACCOUNT_AGE_DAYS;

    // Lockdown check
    let lockdown_key = format!("lockdown:{}", guild_id);
    let lockdown_active: bool = conn.exists(&lockdown_key).await?;

    if lockdown_active {
        return Ok(true);
    }

    // Raid detected: too many joins OR very new accounts
    if recent_joins >= JOIN_THRESHOLD {
        // Start lockdown (5 minutes)
        conn.set_ex::<_, _, ()>(&lockdown_key, "1", 300usize).await?;
        tracing::warn!(
            "LOCKDOWN INITIATED - Guild: {} | {} joins in the last {}s",
            guild_id, recent_joins, TIME_WINDOW
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
    redis: Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>,
    guild_id: serenity::GuildId,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = redis.lock().await;
    let lockdown_key = format!("lockdown:{}", guild_id);
    Ok(conn.exists(&lockdown_key).await?)
}

pub async fn end_lockdown(
    redis: Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>,
    guild_id: serenity::GuildId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = redis.lock().await;
    let lockdown_key = format!("lockdown:{}", guild_id);
    conn.del::<_, ()>(&lockdown_key).await?;
    Ok(())
}
