use poise::serenity_prelude as serenity;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SpamResult {
    pub is_spam: bool,
    pub violation_count: i32,
    pub reason: String,
}

pub async fn check(
    mut redis: redis::aio::ConnectionManager,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    content: &str,
) -> Result<SpamResult, Box<dyn std::error::Error + Send + Sync>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Token bucket key
    let bucket_key = format!("spam:{}:{}", guild_id, user_id);
    let count_key = format!("spam_count:{}:{}", guild_id, user_id);

    // Sliding window: message count in last 3 seconds
    let window_key = format!("spam_window:{}:{}", guild_id, user_id);
    redis.zadd::<_, _, _, ()>(&window_key, now, now).await?;
    redis.zrembyscore::<_, _, _, ()>(&window_key, 0u64, now - 3).await?;
    redis.expire::<_, ()>(&window_key, 10).await?;

    let message_count: i64 = redis.zcard(&window_key).await?;

    // Mention spam check
    let mention_count = content.matches("<@").count();

    // Caps spam check
    let caps_ratio = if content.len() > 10 {
        let upper = content.chars().filter(|c| c.is_uppercase()).count();
        upper as f32 / content.len() as f32
    } else {
        0.0
    };

    // Duplicate message check
    let last_msg_key = format!("last_msg:{}:{}", guild_id, user_id);
    let last_msg: Option<String> = redis.get(&last_msg_key).await?;
    let dup_count_key = format!("dup_count:{}:{}", guild_id, user_id);

    let is_duplicate = last_msg.as_deref() == Some(content);
    if is_duplicate {
        redis.incr::<_, _, ()>(&dup_count_key, 1).await?;
        redis.expire::<_, ()>(&dup_count_key, 30).await?;
    } else {
        redis.set_ex::<_, _, ()>(&last_msg_key, content, 30).await?;
        redis.del::<_, ()>(&dup_count_key).await?;
    }

    let dup_count: i64 = redis.get(&dup_count_key).await.unwrap_or(0);

    // Is it spam?
    let (is_spam, reason) = if message_count > 5 {
        (true, "Message rate too high".to_string())
    } else if mention_count >= 5 {
        (true, "Toplu mention spam".to_string())
    } else if caps_ratio > 0.8 && content.len() > 10 {
        (true, "Caps spam".to_string())
    } else if dup_count >= 3 {
        (true, "Tekrar mesaj spam".to_string())
    } else {
        (false, String::new())
    };

    if !is_spam {
        return Ok(SpamResult {
            is_spam: false,
            violation_count: 0,
            reason: String::new(),
        });
    }

    // Increment violation count
    redis.incr::<_, _, ()>(&count_key, 1).await?;
    redis.expire::<_, ()>(&count_key, 86400).await?; // 24 saat
    let violation_count: i32 = redis.get(&count_key).await.unwrap_or(1);

    Ok(SpamResult {
        is_spam: true,
        violation_count,
        reason,
    })
}
