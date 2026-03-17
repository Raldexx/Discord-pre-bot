use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Bot ayarlarını düzenle
#[poise::command(
    slash_command,
    required_permissions = "ADMINISTRATOR",
    subcommands("otorol", "modrolu", "logkanal", "hosgeldin", "spam", "raid", "linkfiltre")
)]
pub async fn config(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Oto rol ayarla
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn otorol(
    ctx: Context<'_>,
    #[description = "Yeni üyelere verilecek rol"] rol: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, auto_role_id)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET auto_role_id = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(rol.id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    ctx.say(format!("✅ Oto rol **{}** olarak ayarlandı.", rol.name))
        .await?;
    Ok(())
}

/// Moderatör rolü ayarla
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn modrolu(
    ctx: Context<'_>,
    #[description = "Moderatör rolü"] rol: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, mod_role_id)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET mod_role_id = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(rol.id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    ctx.say(format!("✅ Moderatör rolü **{}** olarak ayarlandı.", rol.name))
        .await?;
    Ok(())
}

/// Log kanalı ayarla
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn logkanal(
    ctx: Context<'_>,
    #[description = "Log kanalı"] kanal: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, log_channel_id)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET log_channel_id = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(kanal.id().get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    ctx.say(format!("✅ Log kanalı <#{}> olarak ayarlandı.", kanal.id()))
        .await?;
    Ok(())
}

/// Hoş geldin kanalı ayarla
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn hosgeldin(
    ctx: Context<'_>,
    #[description = "Hoş geldin kanalı"] kanal: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, welcome_channel_id)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET welcome_channel_id = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(kanal.id().get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    ctx.say(format!("✅ Hoş geldin kanalı <#{}> olarak ayarlandı.", kanal.id()))
        .await?;
    Ok(())
}

/// Spam korumasını aç/kapat
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn spam(
    ctx: Context<'_>,
    #[description = "Aktif/Pasif"] aktif: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, spam_protection)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET spam_protection = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(aktif)
    .execute(ctx.data().db.as_ref())
    .await?;

    let durum = if aktif { "✅ Aktif" } else { "❌ Pasif" };
    ctx.say(format!("Spam koruması: **{}**", durum)).await?;
    Ok(())
}

/// Raid korumasını aç/kapat
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn raid(
    ctx: Context<'_>,
    #[description = "Aktif/Pasif"] aktif: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, raid_protection)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET raid_protection = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(aktif)
    .execute(ctx.data().db.as_ref())
    .await?;

    let durum = if aktif { "✅ Aktif" } else { "❌ Pasif" };
    ctx.say(format!("Raid koruması: **{}**", durum)).await?;
    Ok(())
}

/// Link filtresini aç/kapat
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn linkfiltre(
    ctx: Context<'_>,
    #[description = "Aktif/Pasif"] aktif: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO guild_config (guild_id, link_filter)
         VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET link_filter = $2"
    )
    .bind(guild_id.get() as i64)
    .bind(aktif)
    .execute(ctx.data().db.as_ref())
    .await?;

    let durum = if aktif { "✅ Aktif" } else { "❌ Pasif" };
    ctx.say(format!("Link filtresi: **{}**", durum)).await?;
    Ok(())
}
