use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::{info, warn};

pub async fn on_member_join(
    ctx: &serenity::Context,
    member: &serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = member.guild_id;

    // 1. Raid kontrolü
    let is_raid = crate::protection::raid::check_and_update(
        data.redis.as_ref().clone(),
        guild_id,
        member.user.id,
        member.user.created_at(),
    )
    .await?;

    if is_raid {
        warn!("RAID TESPİT EDİLDİ! Guild: {}", guild_id);
        handle_raid(ctx, data, guild_id, member).await?;
        return Ok(());
    }

    // 2. Config'den oto rol ve verification ayarlarını al
    let config = get_guild_config(data, guild_id).await?;

    // 3. Verification açıksa doğrulama kanalına yönlendir
    if config.verification_enabled {
        if let Some(channel_id) = config.verification_channel_id {
            let channel = serenity::ChannelId::new(channel_id as u64);
            let embed = serenity::CreateEmbed::new()
                .title("👋 Hoş Geldin!")
                .description(format!(
                    "Merhaba <@{}>! Sunucuya erişmek için aşağıdaki butona tıkla.",
                    member.user.id
                ))
                .color(serenity::Colour::BLUE);

            let button = serenity::CreateButton::new("verify_button")
                .label("✅ Doğrula")
                .style(serenity::ButtonStyle::Success);

            let components =
                serenity::CreateActionRow::Buttons(vec![button]);

            channel
                .send_message(
                    ctx,
                    serenity::CreateMessage::new()
                        .embed(embed)
                        .components(vec![components]),
                )
                .await?;
        }
    } else if let Some(role_id) = config.auto_role_id {
        // Verification yoksa direkt oto rol ver
        guild_id
            .member(ctx, member.user.id)
            .await?
            .add_role(ctx, serenity::RoleId::new(role_id as u64))
            .await?;
        info!("Oto rol verildi: {} → rol {}", member.user.name, role_id);
    }

    // 4. Karşılama mesajı
    if let Some(welcome_channel_id) = config.welcome_channel_id {
        let channel = serenity::ChannelId::new(welcome_channel_id as u64);
        let embed = serenity::CreateEmbed::new()
            .title("🎉 Yeni Üye!")
            .description(format!(
                "<@{}> sunucuya katıldı! Hoş geldin.",
                member.user.id
            ))
            .thumbnail(
                member
                    .user
                    .avatar_url()
                    .unwrap_or_default(),
            )
            .color(serenity::Colour::FOOYOO);

        channel
            .send_message(ctx, serenity::CreateMessage::new().embed(embed))
            .await?;
    }

    // 5. Log
    log_member_event(ctx, data, guild_id, &member.user, "join").await?;

    Ok(())
}

pub async fn on_member_leave(
    ctx: &serenity::Context,
    guild_id: &serenity::GuildId,
    user: &serenity::User,
    data: &Data,
) -> Result<(), Error> {
    log_member_event(ctx, data, *guild_id, user, "leave").await?;
    Ok(())
}

pub async fn handle_verification(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = interaction.guild_id.unwrap();

    let config = get_guild_config(data, guild_id).await?;

    if let Some(role_id) = config.auto_role_id {
        guild_id
            .member(ctx, interaction.user.id)
            .await?
            .add_role(ctx, serenity::RoleId::new(role_id as u64))
            .await?;

        interaction
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("✅ Doğrulama başarılı! Sunucuya hoş geldin.")
                        .ephemeral(true),
                ),
            )
            .await?;

        info!("Kullanıcı doğrulandı: {}", interaction.user.name);
    }

    Ok(())
}

async fn handle_raid(
    ctx: &serenity::Context,
    data: &Data,
    guild_id: serenity::GuildId,
    member: &serenity::Member,
) -> Result<(), Error> {
    // Yeni üyeyi kick et
    guild_id
        .kick_with_reason(ctx, member.user.id, "Otomatik: Raid koruması")
        .await?;

    // Moderatörleri bildir
    let config = get_guild_config(data, guild_id).await?;
    if let Some(log_channel_id) = config.log_channel_id {
        let channel = serenity::ChannelId::new(log_channel_id as u64);
        let embed = serenity::CreateEmbed::new()
            .title("🚨 RAİD UYARISI!")
            .description(format!(
                "Raid tespiti yapıldı! {} kick edildi.",
                member.user.name
            ))
            .field("Kullanıcı", format!("<@{}>", member.user.id), true)
            .field("Hesap Yaşı", format_account_age(member.user.created_at()), true)
            .color(serenity::Colour::RED)
            .timestamp(serenity::Timestamp::now());

        // Moderatör rolünü mention et
        if let Some(mod_role_id) = config.mod_role_id {
            channel
                .send_message(
                    ctx,
                    serenity::CreateMessage::new()
                        .content(format!("<@&{}> 🚨 Raid tespiti!", mod_role_id))
                        .embed(embed),
                )
                .await?;
        } else {
            channel
                .send_message(ctx, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
    }

    Ok(())
}

async fn log_member_event(
    ctx: &serenity::Context,
    data: &Data,
    guild_id: serenity::GuildId,
    user: &serenity::User,
    event: &str,
) -> Result<(), Error> {
    let config = get_guild_config(data, guild_id).await?;

    if let Some(log_channel_id) = config.log_channel_id {
        let channel = serenity::ChannelId::new(log_channel_id as u64);
        let (title, color) = if event == "join" {
            ("📥 Üye Katıldı", serenity::Colour::FOOYOO)
        } else {
            ("📤 Üye Ayrıldı", serenity::Colour::ORANGE)
        };

        let embed = serenity::CreateEmbed::new()
            .title(title)
            .field("Kullanıcı", format!("{} ({})", user.name, user.id), false)
            .thumbnail(user.avatar_url().unwrap_or_default())
            .color(color)
            .timestamp(serenity::Timestamp::now());

        channel
            .send_message(ctx, serenity::CreateMessage::new().embed(embed))
            .await?;
    }

    Ok(())
}

fn format_account_age(created_at: serenity::Timestamp) -> String {
    let now = chrono::Utc::now();
    let created: chrono::DateTime<chrono::Utc> = created_at.into();
    let days = (now - created).num_days();
    format!("{} gün", days)
}

struct GuildConfig {
    auto_role_id: Option<i64>,
    mod_role_id: Option<i64>,
    log_channel_id: Option<i64>,
    welcome_channel_id: Option<i64>,
    verification_channel_id: Option<i64>,
    verification_enabled: bool,
}

async fn get_guild_config(data: &Data, guild_id: serenity::GuildId) -> Result<GuildConfig, Error> {
    let row = sqlx::query!(
        "SELECT auto_role_id, mod_role_id, log_channel_id, welcome_channel_id,
                verification_channel_id, verification_enabled
         FROM guild_config WHERE guild_id = $1",
        guild_id.get() as i64
    )
    .fetch_optional(data.db.as_ref())
    .await?;

    if let Some(r) = row {
        Ok(GuildConfig {
            auto_role_id: r.auto_role_id,
            mod_role_id: r.mod_role_id,
            log_channel_id: r.log_channel_id,
            welcome_channel_id: r.welcome_channel_id,
            verification_channel_id: r.verification_channel_id,
            verification_enabled: r.verification_enabled.unwrap_or(false),
        })
    } else {
        Ok(GuildConfig {
            auto_role_id: None,
            mod_role_id: None,
            log_channel_id: None,
            welcome_channel_id: None,
            verification_channel_id: None,
            verification_enabled: false,
        })
    }
}
