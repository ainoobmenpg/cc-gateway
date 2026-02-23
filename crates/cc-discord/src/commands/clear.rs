//! /clear command - Clear conversation history (poise implementation)

use tracing::info;

use crate::commands::Data;
use crate::error::Result;

/// Clear conversation history for this channel
#[poise::command(slash_command, rename = "clear")]
pub async fn clear(
    ctx: poise::Context<'_, Data, crate::error::DiscordError>,
) -> Result<()> {
    let channel_id = ctx.channel_id().to_string();
    info!("Clearing conversation history for channel: {}", channel_id);

    // Get shared data
    let data = ctx.data();

    // Clear the session
    let cleared = data.session_store.clear(&channel_id);

    let response = if cleared {
        "会話履歴をクリアしました。"
    } else {
        "このチャンネルには会話履歴がありません。"
    };

    ctx.say(response).await?;

    Ok(())
}
