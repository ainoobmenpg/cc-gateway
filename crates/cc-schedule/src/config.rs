//! スケジュール設定
//!
//! TOML 形式の設定ファイルからスケジュールを読み込みます。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// スケジュール全体の設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleConfig {
    /// スケジュールタスクのリスト
    pub schedules: Vec<ScheduleTask>,
}

/// 個別のスケジュールタスク
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTask {
    /// タスク名
    pub name: String,

    /// cron 形式のスケジュール (例: "0 9 * * *" = 毎日9時)
    pub cron: String,

    /// AI に送信するプロンプト
    pub prompt: String,

    /// 使用するツール（省略時は全ツール使用可）
    #[serde(default)]
    pub tools: Vec<String>,

    /// 結果を送信する Discord チャンネル名（省略可）
    #[serde(default)]
    pub discord_channel: Option<String>,

    /// 有効/無効
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl ScheduleConfig {
    /// TOML ファイルから設定を読み込む
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: ScheduleConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// デフォルトパスから設定を読み込む
    pub fn load_default() -> anyhow::Result<Self> {
        let paths = ["schedule.toml", "config/schedule.toml", ".cc-gateway/schedule.toml"];

        for path in &paths {
            if Path::new(path).exists() {
                return Self::from_file(path);
            }
        }

        // 設定ファイルがない場合は空の設定を返す
        Ok(Self::default())
    }

    /// 有効なタスクのみを返す
    pub fn enabled_tasks(&self) -> Vec<&ScheduleTask> {
        self.schedules.iter().filter(|t| t.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_toml() {
        let toml = r#"
[[schedules]]
name = "毎朝の挨拶"
cron = "0 9 * * *"
prompt = "おはようございます"
"#;
        let config: ScheduleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.schedules.len(), 1);
        assert_eq!(config.schedules[0].name, "毎朝の挨拶");
        assert!(config.schedules[0].enabled); // デフォルトで有効
    }
}
