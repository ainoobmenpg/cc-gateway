//! エラー型定義 (cc-discord)

use thiserror::Error;

/// cc-discord のエラー型
#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("Discord token not set")]
    TokenNotSet,

    #[error("Discord API error: {0}")]
    Api(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serenity error: {0}")]
    SerenityError(#[from] serenity::Error),
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, DiscordError>;
