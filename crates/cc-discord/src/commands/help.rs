//! /help command - Show help information (poise implementation)

use crate::commands::Data;
use crate::error::Result;

/// Show help information about the bot
#[poise::command(slash_command, rename = "help")]
pub async fn help(
    ctx: poise::Context<'_, Data, crate::error::DiscordError>,
) -> Result<()> {
    let response = r#"**Claude Code Gateway Bot**

このボットはDiscordを通じてClaude AIにアクセスするためのゲートウェイです。

**使い方:**

1. **メンション**: ボットをメンションしてメッセージを送ると応答します
   例: `@ClaudeBot こんにちは！`

2. **DM**: ダイレクトメッセージでも会話できます

3. **返信**: ボットのメッセージに返信すると会話が続きます

**Slash Commands:**

- `/ask <question>` - Claudeに質問する
- `/clear` - 現在のチャンネルの会話履歴をクリアする
- `/help` - このヘルプを表示

**注意事項:**
- 管理者のみ使用可能です（設定で制御）
- メッセージは2000文字で制限されます
"#;

    ctx.say(response).await?;

    Ok(())
}
