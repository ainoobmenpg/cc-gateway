//! エラー型定義 (cc-schedule)

use thiserror::Error;

/// cc-schedule のエラー型
#[derive(Error, Debug)]
pub enum ScheduleError {
    #[error("cron パースエラー: {0}")]
    CronParse(String),

    #[error("設定ファイル読み込みエラー: {0}")]
    ConfigLoad(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Core error: {0}")]
    Core(#[from] cc_core::Error),

    #[error("Cron error: {0}")]
    CronError(#[from] cron::error::Error),
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, ScheduleError>;
