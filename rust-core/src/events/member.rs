use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::info;

pub async fn on_member_join(
    ctx: &serenity::Context,
    member: &serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = member.guild_id;

    let is_raid = crate::protection::raid::check_and_update(
        data.redis.as_ref().clone(),
        guild_id,
        member.user.id,
        member.user.created_at(),
    ).await?;

    if is_raid {
        handle_raid(ctx, data, guild_id, member).await?;
        return Ok(());
    }

    let config = get_guild_config(data, guild_id).await?;

    if config.verification_enabled {
        if let Some(channel_id) = config.verification_channel_id {
            let channel = serenity::ChannelId::new(channel_id as u64);
            let embed = serenity::CreateEmbed::new()
                .title("👋 Welcome!")
                .description(format!(
                    "Hey <@{}>! Click the button below to access the server.",
                    member.user.id
                ))
                .color(serenity::Colour::BLUE);

            let button = serenity::CreateButton::new("verify_button")
                .label("✅ Verify")
                .style(serenity::ButtonStyle::Success);

            channel.send_message(ctx,
                serenity::CreateMessage::new()
                    .embed(embed)
                    .components(vec![serenity::CreateActionRow::Buttons(vec![button])]),
            ).await?;
        }
    } else if let Some(role_id) = config.auto_role_id {
        guild_id.member(ctx, member.user.id).await?
            .add_role(ctx, serenity::RoleId::new(role_id as u64)).await?;
        info!("Auto role assigned: {} → role {}", member.user.name, role_id);
    }

    if let Some(welcome_channel_id) = config.welcome_channel_id {
        let channel = serenity::ChannelId::new(welcome_channel_id as u64);
        let embed = serenity::CreateEmbed::new()
            .title("🎉 New Member!")
            .description(format!("<@{}> just joined the server! Welcome.", member.user.id))
            .thumbnail(member.user.avatar_url().unwrap_or_default())
            .color(serenity::Colour::FOOYOO);
        channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    }

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
        guild_id.member(ctx, interaction.user.id).await?
            .add_role(ctx, serenity::RoleId::new(role_id as u64)).await?;

        interaction.create_response(ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("✅ Verified! Welcome to the server.")
                    .ephemeral(true),
            ),
        ).await?;
    }
    Ok(())
}

async fn handle_raid(
    ctx: &serenity::Context,
    data: &Data,
    guild_id: serenity::GuildId,
    member: &serenity::Member,
) -> Result<(), Error> {
    guild_id.kick_with_reason(ctx, member.user.id, "Auto: Raid protection").await?;

    let config = get_guild_config(data, guild_id).await?;
    if let Some(log_channel_id) = config.log_channel_id {
        let channel = serenity::ChannelId::new(log_channel_id as u64);
        let embed = serenity::CreateEmbed::new()
            .title("🚨 RAID ALERT!")
            .description(format!("Raid detected! {} was kicked.", member.user.name))
            .field("User", format!("<@{}>", member.user.id), true)
            .field("Account Age", format_account_age(member.user.created_at()), true)
            .color(serenity::Colour::RED)
            .timestamp(serenity::Timestamp::now());

        let content = config.mod_role_id
            .map(|id| format!("<@&{}> 🚨 Raid detected!", id))
            .unwrap_or_default();

        channel.send_message(ctx,
            serenity::CreateMessage::new().content(content).embed(embed),
        ).await?;
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
            ("📥 Member Joined", serenity::Colour::FOOYOO)
        } else {
            ("📤 Member Left", serenity::Colour::ORANGE)
        };
        let embed = serenity::CreateEmbed::new()
            .title(title)
            .field("User", format!("{} ({})", user.name, user.id), false)
            .thumbnail(user.avatar_url().unwrap_or_default())
            .color(color)
            .timestamp(serenity::Timestamp::now());
        channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    }
    Ok(())
}

fn format_account_age(created_at: serenity::Timestamp) -> String {
    let now = chrono::Utc::now();
    let created: chrono::DateTime<chrono::Utc> = created_at.into();
    format!("{} days", (now - created).num_days())
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
            auto_role_id: None, mod_role_id: None, log_channel_id: None,
            welcome_channel_id: None, verification_channel_id: None,
            verification_enabled: false,
        })
    }
}
