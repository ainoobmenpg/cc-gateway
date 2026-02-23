//! エラー型定義 (cc-discord)

use poise::serenity_prelude as serenity;
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
    SerenityError(Box<serenity::Error>),
}

// Manual From implementation to box the error
impl From<serenity::Error> for DiscordError {
    fn from(error: serenity::Error) -> Self {
        DiscordError::SerenityError(Box::new(error))
    }
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, DiscordError>;
