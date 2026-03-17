use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Botu hızlıca yapılandır
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    ctx.defer_ephemeral().await?;

    // Mevcut config kontrol et
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT guild_id FROM guild_config WHERE guild_id = $1"
    )
    .bind(guild_id.get() as i64)
    .fetch_optional(ctx.data().db.as_ref())
    .await?;

    if existing.is_some() {
        ctx.say("⚙️ Bu sunucu zaten yapılandırılmış! `/config` ile ayarları düzenleyebilirsin.").await?;
        return Ok(());
    }

    // Temel kanalları oluştur
    let guild = guild_id.to_partial_guild(ctx).await?;

    // Log kanalı oluştur
    let log_channel = guild
        .create_channel(
            ctx,
            serenity::CreateChannel::new("🔒│bot-logs")
                .kind(serenity::ChannelType::Text)
                .topic("Bot log kanalı - otomatik oluşturuldu"),
        )
        .await?;

    // Verification kanalı oluştur
    let verify_channel = guild
        .create_channel(
            ctx,
            serenity::CreateChannel::new("✅│doğrulama")
                .kind(serenity::ChannelType::Text)
                .topic("Sunucuya erişmek için burada doğrulama yap"),
        )
        .await?;

    // Rol kanalı oluştur
    let roles_channel = guild
        .create_channel(
            ctx,
            serenity::CreateChannel::new("🎭│roller")
                .kind(serenity::ChannelType::Text)
                .topic("Rol seçim kanalı"),
        )
        .await?;

    // DB'ye kaydet
    sqlx::query(
        "INSERT INTO guild_config
         (guild_id, log_channel_id, verification_channel_id, roles_channel_id,
          verification_enabled, spam_protection, raid_protection, link_filter)
         VALUES ($1, $2, $3, $4, true, true, true, true)
         ON CONFLICT (guild_id) DO NOTHING"
    )
    .bind(guild_id.get() as i64)
    .bind(log_channel.id.get() as i64)
    .bind(verify_channel.id.get() as i64)
    .bind(roles_channel.id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    let embed = serenity::CreateEmbed::new()
        .title("✅ Kurulum Tamamlandı!")
        .description("Bot başarıyla yapılandırıldı. Aşağıdaki kanallar oluşturuldu:")
        .field("📋 Log Kanalı", format!("<#{}>", log_channel.id), false)
        .field("✅ Doğrulama Kanalı", format!("<#{}>", verify_channel.id), false)
        .field("🎭 Rol Kanalı", format!("<#{}>", roles_channel.id), false)
        .field(
            "⚙️ Aktif Özellikler",
            "✅ Spam Koruması\n✅ Raid Koruması\n✅ Link Filtresi\n✅ Doğrulama Sistemi",
            false,
        )
        .field(
            "📌 Sonraki Adımlar",
            "• `/config otorol` ile oto rol ayarla\n• `/config modrolu` ile mod rolünü belirle\n• `/rolpanel` ile rol paneli oluştur",
            false,
        )
        .color(serenity::Colour::FOOYOO)
        .timestamp(serenity::Timestamp::now());

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
