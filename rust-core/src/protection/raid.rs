use poise::serenity_prelude as serenity;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const JOIN_THRESHOLD: i64 = 10;
const TIME_WINDOW: u64 = 10;
const MIN_ACCOUNT_AGE_DAYS: i64 = 7;

pub async fn check_and_update(
    redis: Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    account_created: serenity::Timestamp,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn: tokio::sync::MutexGuard<redis::aio::MultiplexedConnection> =
        redis.lock().await;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let join_key = format!("joins:{}", guild_id);
    conn.zadd::<_, _, _, ()>(&join_key, now, format!("{}:{}", user_id, now))
        .await?;
    conn.zrembyscore::<_, _, _, ()>(&join_key, 0u64, now - TIME_WINDOW)
        .await?;
    conn.expire::<_, ()>(&join_key, 60).await?;
    let recent_joins: i64 = conn.zcard(&join_key).await?;

    let created_secs = account_created.unix_timestamp();
    let now_secs = chrono::Utc::now().timestamp();
    let account_age_days = (now_secs - created_secs) / 86400;
    let is_new_account = account_age_days < MIN_ACCOUNT_AGE_DAYS;

    let lockdown_key = format!("lockdown:{}", guild_id);
    let lockdown_active: bool = conn.exists(&lockdown_key).await?;

    if lockdown_active {
        return Ok(true);
    }

    if recent_joins >= JOIN_THRESHOLD {
        conn.set_ex::<_, _, ()>(&lockdown_key, "1", 300u64).await?;
        tracing::warn!(
            "LOCKDOWN INITIATED - Guild: {} | {} joins in {}s",
            guild_id,
            recent_joins,
            TIME_WINDOW
        );
        return Ok(true);
    }

    if is_new_account && recent_joins >= JOIN_THRESHOLD / 2 {
        return Ok(true);
    }

    Ok(false)
}
