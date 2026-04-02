use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Edit bot settings
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    subcommands("autorole", "modrole", "logchannel", "welcome", "spam", "raid", "linkfilter")
)]
pub async fn config(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set auto role for new members
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn autorole(
    ctx: Context<'_>,
    #[description = "Role to give new members"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, auto_role_id) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET auto_role_id = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(role.id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!("✅ Auto role set to **{}**.", role.name))
        .await?;
    Ok(())
}

/// Set moderator role
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn modrole(
    ctx: Context<'_>,
    #[description = "Moderator role"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, mod_role_id) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET mod_role_id = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(role.id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!("✅ Mod role set to **{}**.", role.name))
        .await?;
    Ok(())
}

/// Set log channel
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn logchannel(
    ctx: Context<'_>,
    #[description = "Log channel"] channel: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, log_channel_id) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET log_channel_id = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(channel.id().get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!("✅ Log channel set to <#{}>.", channel.id()))
        .await?;
    Ok(())
}

/// Set welcome channel
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn welcome(
    ctx: Context<'_>,
    #[description = "Welcome channel"] channel: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, welcome_channel_id) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET welcome_channel_id = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(channel.id().get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!("✅ Welcome channel set to <#{}>.", channel.id()))
        .await?;
    Ok(())
}

/// Toggle spam protection
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn spam(
    ctx: Context<'_>,
    #[description = "Enable or disable"] enabled: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, spam_protection) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET spam_protection = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(enabled)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!(
        "Spam protection: **{}**",
        if enabled { "✅ Enabled" } else { "❌ Disabled" }
    ))
    .await?;
    Ok(())
}

/// Toggle raid protection
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn raid(
    ctx: Context<'_>,
    #[description = "Enable or disable"] enabled: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, raid_protection) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET raid_protection = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(enabled)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!(
        "Raid protection: **{}**",
        if enabled { "✅ Enabled" } else { "❌ Disabled" }
    ))
    .await?;
    Ok(())
}

/// Toggle link filter
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn linkfilter(
    ctx: Context<'_>,
    #[description = "Enable or disable"] enabled: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    sqlx::query(
        "INSERT INTO guild_config (guild_id, link_filter) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET link_filter = $2",
    )
    .bind(guild_id.get() as i64)
    .bind(enabled)
    .execute(ctx.data().db.as_ref())
    .await?;
    ctx.say(format!(
        "Link filter: **{}**",
        if enabled { "✅ Enabled" } else { "❌ Disabled" }
    ))
    .await?;
    Ok(())
}
