//! /help command - Show help information

use serenity::all::{CommandInteraction, CreateCommand};

/// Register the /help command
pub fn register_help_command(command: CreateCommand) -> CreateCommand {
    command
        .name("help")
        .description("Show help information about the bot")
}

/// Run the /help command
pub async fn run(_interaction: &CommandInteraction) -> String {
    r#"**Claude Code Gateway Bot**

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
"#.to_string()
}
