pub mod member;
pub mod message;

use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::info;

pub async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }
            message::on_message(ctx, new_message, data).await?;
        }

        serenity::FullEvent::GuildMemberAddition { new_member } => {
            info!("New member: {} ({})", new_member.user.name, new_member.guild_id);
            member::on_member_join(ctx, new_member, data).await?;
        }

        serenity::FullEvent::GuildMemberRemoval {
            guild_id, user, ..
        } => {
            member::on_member_leave(ctx, guild_id, user, data).await?;
        }

        serenity::FullEvent::InteractionCreate { interaction } => {
            if let serenity::Interaction::Component(component) = interaction {
                handle_component(ctx, component, data).await?;
            }
        }

        serenity::FullEvent::Ready { data_about_bot } => {
            info!("Bot ready: {}", data_about_bot.user.name);
        }

        _ => {}
    }
    Ok(())
}

async fn handle_component(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Verification butonu
    if custom_id == "verify_button" {
        member::handle_verification(ctx, interaction, data).await?;
        return Ok(());
    }

    // Role buttons
    if custom_id.starts_with("role_") {
        let role_id_str = custom_id.strip_prefix("role_").unwrap();
        if let Ok(role_id) = role_id_str.parse::<u64>() {
            handle_role_toggle(ctx, interaction, serenity::RoleId::new(role_id)).await?;
        }
    }

    Ok(())
}

async fn handle_role_toggle(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    role_id: serenity::RoleId,
) -> Result<(), Error> {
    let guild_id = interaction.guild_id.unwrap();
    let member = guild_id.member(ctx, interaction.user.id).await?;

    let response = if member.roles.contains(&role_id) {
        guild_id
            .member(ctx, interaction.user.id)
            .await?
            .remove_role(ctx, role_id)
            .await?;
        "✅ Role removed."
    } else {
        guild_id
            .member(ctx, interaction.user.id)
            .await?
            .add_role(ctx, role_id)
            .await?;
        "✅ Role assigned."
    };

    interaction
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(response)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
