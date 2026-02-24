//! Telegram bot commands

use std::sync::Arc;
use teloxide::prelude::*;
use tracing::info;

use cc_core::ClaudeClient;

use crate::error::Result;
use crate::session::InMemorySessionStore;

/// Alias for cc_core::Message to avoid conflict with teloxide::types::Message
type CoreMessage = cc_core::Message;

/// Bot state shared across commands
pub struct BotState {
    pub claude_client: Arc<ClaudeClient>,
    pub session_store: Arc<InMemorySessionStore>,
    pub admin_user_ids: Vec<i64>,
}

/// Handle /ask command
pub async fn handle_ask(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    question: String,
) -> Result<()> {
    let user_id = msg.chat.id.0;
    let chat_id = msg.chat.id;

    info!("Processing /ask from user {}: {}", user_id, question);

    // Check admin permission
    if !state.admin_user_ids.is_empty() && !state.admin_user_ids.contains(&user_id) {
        bot.send_message(chat_id, "⚠️ 認証エラー: このボットを使用する権限がありません。")
            .await?;
        return Ok(());
    }

    if question.trim().is_empty() {
        bot.send_message(chat_id, "質問を入力してください。使い方: /ask <質問>")
            .await?;
        return Ok(());
    }

    // Get or create session
    let session_key = chat_id.to_string();
    let session = state.session_store.get_or_create(&session_key).await;

    // Build message history
    let mut messages: Vec<CoreMessage> = session.messages.clone();
    messages.push(CoreMessage::user(&question));

    // Build request
    let mut request_builder = state
        .claude_client
        .request_builder()
        .system("You are a helpful assistant. Respond in the same language as the user's question. Be concise and helpful.")
        .max_tokens(2048);

    // Add conversation history (limit to last 20 messages)
    let history_start = messages.len().saturating_sub(20);
    for message in messages.into_iter().skip(history_start) {
        request_builder = request_builder.message(message);
    }

    let request = request_builder.build();

    // Send "typing" action
    bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
        .await?;

    // Call Claude API
    let response_text = match state.claude_client.messages(request).await {
        Ok(response) => {
            let text = response
                .content
                .iter()
                .filter_map(|c| {
                    if let cc_core::MessageContent::Text { text } = c {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Update session
            state
                .session_store
                .add_message(&session_key, CoreMessage::user(&question))
                .await;
            state
                .session_store
                .add_message(&session_key, CoreMessage::assistant(&text))
                .await;

            // Truncate if too long for Telegram (4096 char limit)
            if text.len() > 4000 {
                format!("{}\n\n...(truncated)", &text[..4000])
            } else {
                text
            }
        }
        Err(e) => format!("エラーが発生しました: {}", e),
    };

    // Send response (split if needed)
    if response_text.len() > 4096 {
        for chunk in response_text.as_bytes().chunks(4000) {
            let chunk_text = String::from_utf8_lossy(chunk).to_string();
            bot.send_message(chat_id, &chunk_text).await?;
        }
    } else {
        bot.send_message(chat_id, &response_text).await?;
    }

    Ok(())
}

/// Handle /clear command
pub async fn handle_clear(bot: Bot, msg: Message, state: Arc<BotState>) -> Result<()> {
    let user_id = msg.chat.id.0;
    let chat_id = msg.chat.id;

    // Check admin permission
    if !state.admin_user_ids.is_empty() && !state.admin_user_ids.contains(&user_id) {
        bot.send_message(chat_id, "⚠️ 認証エラー: このボットを使用する権限がありません。")
            .await?;
        return Ok(());
    }

    let session_key = chat_id.to_string();
    state.session_store.clear(&session_key).await;

    bot.send_message(chat_id, "✅ 会話履歴をクリアしました。")
        .await?;

    info!("Cleared session for chat {}", chat_id);
    Ok(())
}

/// Handle /help command
pub async fn handle_help(bot: Bot, msg: Message) -> Result<()> {
    let help_text = r#"🤖 cc-gateway Telegram Bot

使い方:
/ask <質問> - Claude に質問する
/clear - 会話履歴をクリア
/help - このヘルプを表示

例:
/ask 今日の天気は？
/ask 前の会話を覚えてる？

powered by cc-gateway"#;

    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}
