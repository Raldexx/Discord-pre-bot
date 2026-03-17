use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Bot hakkında bilgi
#[poise::command(slash_command)]
pub async fn botinfo(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("🤖 Bot Bilgisi")
        .field("Versiyon", "0.1.0", true)
        .field("Dil", "Rust + Python", true)
        .field("Kütüphane", "Serenity + Poise", true)
        .field(
            "Özellikler",
            "🛡️ Spam Koruması\n🚨 Raid Koruması\n🔗 Link Filtresi\n🤖 AI Moderasyon\n🎭 Oto Rol\n✅ Verification\n📋 Loglama",
            false,
        )
        .field("Kaynak Kod", "[GitHub](https://github.com/your-repo)", true)
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Sunucu istatistikleri
#[poise::command(slash_command)]
pub async fn sunucuinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = guild_id.to_partial_guild(ctx).await?;

    // DB'den istatistikleri al
    let warn_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM warnings WHERE guild_id = $1"
    )
    .bind(guild_id.get() as i64)
    .fetch_one(ctx.data().db.as_ref())
    .await
    .unwrap_or(0);

    let embed = serenity::CreateEmbed::new()
        .title(format!("📊 {}", guild.name))
        .field("Sunucu ID", guild_id.to_string(), true)
        .field("Sahip", format!("<@{}>", guild.owner_id), true)
        .field("Toplam Uyarı", warn_count.to_string(), true)
        .thumbnail(guild.icon_url().unwrap_or_default())
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Uyarı geçmişini gör
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn uyarilar(
    ctx: Context<'_>,
    #[description = "Kullanıcı"] kullanici: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let warnings = sqlx::query!(
        "SELECT reason, created_at FROM warnings
         WHERE user_id = $1 AND guild_id = $2
         ORDER BY created_at DESC LIMIT 10",
        kullanici.id.get() as i64,
        guild_id.get() as i64
    )
    .fetch_all(ctx.data().db.as_ref())
    .await?;

    if warnings.is_empty() {
        ctx.say(format!("✅ {} kullanıcısının hiç uyarısı yok.", kullanici.name))
            .await?;
        return Ok(());
    }

    let uyari_listesi = warnings
        .iter()
        .enumerate()
        .map(|(i, w)| {
            format!(
                "**{}**. {} — `{}`",
                i + 1,
                w.reason,
                w.created_at.format("%d/%m/%Y")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title(format!("⚠️ {} — Uyarı Geçmişi", kullanici.name))
        .description(uyari_listesi)
        .field("Toplam", warnings.len().to_string(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
