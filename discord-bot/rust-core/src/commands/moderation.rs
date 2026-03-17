use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Kullanıcıya uyarı ver
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "Uyarılacak kullanıcı"] kullanici: serenity::User,
    #[description = "Uyarı nedeni"] neden: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    sqlx::query(
        "INSERT INTO warnings (user_id, guild_id, reason, moderator_id, created_at)
         VALUES ($1, $2, $3, $4, NOW())"
    )
    .bind(kullanici.id.get() as i64)
    .bind(guild_id.get() as i64)
    .bind(&neden)
    .bind(ctx.author().id.get() as i64)
    .execute(ctx.data().db.as_ref())
    .await?;

    let warn_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM warnings WHERE user_id = $1 AND guild_id = $2"
    )
    .bind(kullanici.id.get() as i64)
    .bind(guild_id.get() as i64)
    .fetch_one(ctx.data().db.as_ref())
    .await?;

    // Kullanıcıya DM at
    let dm_embed = serenity::CreateEmbed::new()
        .title("⚠️ Uyarı Aldınız")
        .field("Sunucu", guild_id.to_string(), true)
        .field("Neden", &neden, false)
        .field("Toplam Uyarı", warn_count.to_string(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    kullanici
        .dm(
            ctx,
            serenity::CreateMessage::new().embed(dm_embed),
        )
        .await
        .ok(); // DM kapalıysa sessizce devam et

    let embed = serenity::CreateEmbed::new()
        .title("⚠️ Uyarı Verildi")
        .field("Kullanıcı", format!("{} ({})", kullanici.name, kullanici.id), true)
        .field("Neden", &neden, false)
        .field("Toplam Uyarı", warn_count.to_string(), true)
        .field("Moderatör", ctx.author().name.clone(), true)
        .color(serenity::Colour::ORANGE)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Kullanıcıyı sustur
#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "Susturulacak kullanıcı"] kullanici: serenity::User,
    #[description = "Süre (dakika)"] dakika: u32,
    #[description = "Neden"] neden: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = neden.unwrap_or_else(|| "Neden belirtilmedi".to_string());

    let until = chrono::Utc::now() + chrono::Duration::minutes(dakika as i64);

    guild_id
        .edit_member(
            ctx,
            kullanici.id,
            serenity::EditMember::new().disable_communication_until(until.into()),
        )
        .await?;

    let embed = serenity::CreateEmbed::new()
        .title("🔇 Kullanıcı Susturuldu")
        .field("Kullanıcı", format!("{} ({})", kullanici.name, kullanici.id), true)
        .field("Süre", format!("{} dakika", dakika), true)
        .field("Neden", &reason, false)
        .field("Moderatör", ctx.author().name.clone(), true)
        .color(serenity::Colour::RED)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Kullanıcıyı banla
#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Banlanacak kullanıcı"] kullanici: serenity::User,
    #[description = "Neden"] neden: Option<String>,
    #[description = "Silinecek mesaj günü (0-7)"] mesaj_gun: Option<u8>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = neden.unwrap_or_else(|| "Neden belirtilmedi".to_string());
    let delete_days = mesaj_gun.unwrap_or(0).min(7);

    guild_id
        .ban_with_reason(ctx, kullanici.id, delete_days, &reason)
        .await?;

    let embed = serenity::CreateEmbed::new()
        .title("🔨 Kullanıcı Banlandı")
        .field("Kullanıcı", format!("{} ({})", kullanici.name, kullanici.id), true)
        .field("Neden", &reason, false)
        .field("Moderatör", ctx.author().name.clone(), true)
        .color(serenity::Colour::RED)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Ban kaldır
#[poise::command(slash_command, required_permissions = "BAN_MEMBERS")]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "Kullanıcı ID"] kullanici_id: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let user_id: u64 = kullanici_id.parse()?;

    guild_id
        .unban(ctx, serenity::UserId::new(user_id))
        .await?;

    ctx.say(format!("✅ <@{}> kullanıcısının banı kaldırıldı.", user_id))
        .await?;
    Ok(())
}

/// Mesaj temizle
#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
pub async fn clear(
    ctx: Context<'_>,
    #[description = "Silinecek mesaj sayısı (1-100)"] adet: u8,
) -> Result<(), Error> {
    let adet = adet.clamp(1, 100);

    let messages = ctx
        .channel_id()
        .messages(ctx, serenity::GetMessages::new().limit(adet))
        .await?;

    let message_ids: Vec<serenity::MessageId> = messages.iter().map(|m| m.id).collect();
    let count = message_ids.len();

    ctx.channel_id()
        .delete_messages(ctx, &message_ids)
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!("🗑️ {} mesaj silindi.", count))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Manuel lockdown (raid sırasında)
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn lockdown(
    ctx: Context<'_>,
    #[description = "Aç veya Kapat"] durum: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let mut redis = ctx.data().redis.as_ref().clone();

    let lockdown_key = format!("lockdown:{}", guild_id);

    match durum.to_lowercase().as_str() {
        "ac" | "aç" | "on" => {
            redis::AsyncCommands::set_ex::<_, _, ()>(&mut redis, &lockdown_key, "1", 3600).await?;
            ctx.say("🔒 **LOCKDOWN AKTİF!** Yeni üye girişleri engellendi. Kapatmak için `/lockdown kapat` kullan.").await?;
        }
        "kapat" | "off" => {
            redis::AsyncCommands::del::<_, ()>(&mut redis, &lockdown_key).await?;
            ctx.say("🔓 Lockdown kaldırıldı. Sunucu normale döndü.").await?;
        }
        _ => {
            ctx.say("❌ Geçersiz seçenek. `ac` veya `kapat` kullan.").await?;
        }
    }

    Ok(())
}
