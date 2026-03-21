use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Quick setup for the bot
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    ctx.defer_ephemeral().await?;

    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT guild_id FROM guild_config WHERE guild_id = $1"
    )
    .bind(guild_id.get() as i64)
    .fetch_optional(ctx.data().db.as_ref())
    .await?;

    if existing.is_some() {
        ctx.say("⚙️ This server is already configured! Use `/config` to edit settings.").await?;
        return Ok(());
    }

    let guild = guild_id.to_partial_guild(ctx).await?;

    let log_channel = guild.create_channel(ctx,
        serenity::CreateChannel::new("🔒│bot-logs")
            .kind(serenity::ChannelType::Text)
            .topic("Bot log channel — auto created"),
    ).await?;

    let verify_channel = guild.create_channel(ctx,
        serenity::CreateChannel::new("✅│verify")
            .kind(serenity::ChannelType::Text)
            .topic("Verify here to access the server"),
    ).await?;

    let roles_channel = guild.create_channel(ctx,
        serenity::CreateChannel::new("🎭│roles")
            .kind(serenity::ChannelType::Text)
            .topic("Pick your roles here"),
    ).await?;

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
        .title("✅ Setup Complete!")
        .description("Bot configured successfully. The following channels were created:")
        .field("📋 Log Channel", format!("<#{}>", log_channel.id), false)
        .field("✅ Verification Channel", format!("<#{}>", verify_channel.id), false)
        .field("🎭 Roles Channel", format!("<#{}>", roles_channel.id), false)
        .field("⚙️ Active Features",
            "✅ Spam Protection\n✅ Raid Protection\n✅ Link Filter\n✅ Verification System", false)
        .field("📌 Next Steps",
            "• `/config autorole` — set auto role\n• `/config modrole` — set mod role\n• `/rolepanel` — create role panel", false)
        .color(serenity::Colour::FOOYOO)
        .timestamp(serenity::Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
