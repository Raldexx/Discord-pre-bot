use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Rol seçim paneli oluştur
#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
pub async fn rolpanel(
    ctx: Context<'_>,
    #[description = "Panel başlığı"] baslik: String,
    #[description = "Açıklama"] aciklama: Option<String>,
    #[description = "Rol 1"] rol1: serenity::Role,
    #[description = "Rol 2"] rol2: Option<serenity::Role>,
    #[description = "Rol 3"] rol3: Option<serenity::Role>,
    #[description = "Rol 4"] rol4: Option<serenity::Role>,
    #[description = "Rol 5"] rol5: Option<serenity::Role>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let mut roller = vec![rol1];
    if let Some(r) = rol2 { roller.push(r); }
    if let Some(r) = rol3 { roller.push(r); }
    if let Some(r) = rol4 { roller.push(r); }
    if let Some(r) = rol5 { roller.push(r); }

    let rol_listesi = roller
        .iter()
        .map(|r| format!("• <@&{}>", r.id))
        .collect::<Vec<_>>()
        .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title(&baslik)
        .description(format!(
            "{}\n\n**Mevcut Roller:**\n{}",
            aciklama.as_deref().unwrap_or("Aşağıdaki butonlarla rol alabilir veya bırakabilirsin."),
            rol_listesi
        ))
        .color(serenity::Colour::BLURPLE)
        .timestamp(serenity::Timestamp::now());

    // Her rol için buton oluştur
    let buttons: Vec<serenity::CreateButton> = roller
        .iter()
        .map(|r| {
            serenity::CreateButton::new(format!("role_{}", r.id))
                .label(&r.name)
                .style(serenity::ButtonStyle::Secondary)
        })
        .collect();

    // Butonları 5'li satırlara böl
    let chunks: Vec<Vec<serenity::CreateButton>> = buttons
        .chunks(5)
        .map(|c| c.to_vec())
        .collect();

    let components: Vec<serenity::CreateActionRow> = chunks
        .into_iter()
        .map(serenity::CreateActionRow::Buttons)
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
            .content("✅ Rol paneli oluşturuldu!")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
