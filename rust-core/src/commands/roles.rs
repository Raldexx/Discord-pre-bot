use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Create a role selection panel
#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
pub async fn rolepanel(
    ctx: Context<'_>,
    #[description = "Panel title"] title: String,
    #[description = "Description"] description: Option<String>,
    #[description = "Role 1"] role1: serenity::Role,
    #[description = "Role 2"] role2: Option<serenity::Role>,
    #[description = "Role 3"] role3: Option<serenity::Role>,
    #[description = "Role 4"] role4: Option<serenity::Role>,
    #[description = "Role 5"] role5: Option<serenity::Role>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mut roles = vec![role1];
    if let Some(r) = role2 { roles.push(r); }
    if let Some(r) = role3 { roles.push(r); }
    if let Some(r) = role4 { roles.push(r); }
    if let Some(r) = role5 { roles.push(r); }

    let role_list = roles
        .iter()
        .map(|r| format!("• <@&{}>", r.id))
        .collect::<Vec<_>>()
        .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title(&title)
        .description(format!(
            "{}\n\n**Available Roles:**\n{}",
            description
                .as_deref()
                .unwrap_or("Use the buttons below to get or remove roles."),
            role_list
        ))
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    let buttons: Vec<serenity::CreateButton> = roles
        .iter()
        .map(|r| {
            serenity::CreateButton::new(format!("role_{}", r.id))
                .label(&r.name)
                .style(serenity::ButtonStyle::Secondary)
        })
        .collect();

    let components: Vec<serenity::CreateActionRow> = buttons
        .chunks(5)
        .map(|c: &[serenity::CreateButton]| {
            serenity::CreateActionRow::Buttons(c.to_vec())
        })
        .collect();

    ctx.channel_id()
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .embed(embed)
                .components(components),
        )
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content("✅ Role panel created!")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
