//! スケジューラー
//!
//! cron スケジュールに基づいてタスクを実行します。

use crate::config::{ScheduleConfig, ScheduleTask};
use cc_core::{ClaudeClient, ToolManager};
use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// スケジューラーのハンドル
pub struct SchedulerHandle {
    /// スケジューラータスクの終了送信
    shutdown_tx: broadcast::Sender<()>,
    /// 実行中のタスクハンドル
    handle: JoinHandle<()>,
}

impl SchedulerHandle {
    /// スケジューラーを停止
    pub async fn stop(self) {
        let _ = self.shutdown_tx.send(());
        let _ = self.handle.await;
    }
}

/// スケジュール実行結果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScheduleResult {
    /// タスク名
    pub task_name: String,
    /// 実行時刻
    pub executed_at: DateTime<Utc>,
    /// AI の応答
    pub response: String,
    /// 成功/失敗
    pub success: bool,
}

/// スケジューラー
pub struct Scheduler {
    config: ScheduleConfig,
    client: ClaudeClient,
    tool_manager: Arc<ToolManager>,
    system_prompt: String,
}

impl Scheduler {
    /// 新しいスケジューラーを作成
    pub fn new(
        config: ScheduleConfig,
        client: ClaudeClient,
        tool_manager: Arc<ToolManager>,
    ) -> Self {
        Self {
            config,
            client,
            tool_manager,
            system_prompt: "あなたはスケジュールされたタスクを実行する AI アシスタントです。\
                指示に従って作業を行い、結果を報告してください。"
                .to_string(),
        }
    }

    /// システムプロンプトを設定
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    /// スケジューラーを開始
    pub fn start(self) -> SchedulerHandle {
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
        let shutdown_tx_clone = shutdown_tx.clone();

        let handle = tokio::spawn(async move {
            info!("スケジューラーを開始しました ({} タスク)", self.config.schedules.len());

            // 各タスクを別々のタスクで実行
            let mut task_handles = Vec::new();

            for task in self.config.enabled_tasks() {
                let task = task.clone();
                let client = self.client.clone();
                let tool_manager = Arc::clone(&self.tool_manager);
                let system_prompt = self.system_prompt.clone();
                let mut rx = shutdown_rx.resubscribe();

                let handle = tokio::spawn(async move {
                    run_schedule_task(task, client, tool_manager, system_prompt, &mut rx).await;
                });

                task_handles.push(handle);
            }

            // 全タスクが終了するまで待機
            for handle in task_handles {
                let _ = handle.await;
            }

            info!("スケジューラーを停止しました");
        });

        SchedulerHandle {
            shutdown_tx: shutdown_tx_clone,
            handle,
        }
    }
}

/// 個別のスケジュールタスクを実行
async fn run_schedule_task(
    task: ScheduleTask,
    client: ClaudeClient,
    tool_manager: Arc<ToolManager>,
    system_prompt: String,
    shutdown_rx: &mut broadcast::Receiver<()>,
) {
    // cron スケジュールをパース
    let schedule = match parse_cron(&task.cron) {
        Ok(s) => s,
        Err(e) => {
            error!(task = %task.name, "cron パースエラー: {}", e);
            return;
        }
    };

    info!(task = %task.name, cron = %task.cron, "スケジュールタスクを開始");

    loop {
        // 次の実行時刻を取得
        let now = Utc::now();
        let next = match schedule.upcoming(Utc).next() {
            Some(t) => t,
            None => {
                warn!(task = %task.name, "次の実行時刻を取得できません");
                break;
            }
        };

        let delay = (next - now).to_std().unwrap_or(Duration::ZERO);
        info!(
            task = %task.name,
            next = %next.format("%Y-%m-%d %H:%M:%S"),
            "次回実行まで待機中"
        );

        // 実行時刻まで待機（シャットダウン確認付き）
        tokio::select! {
            _ = tokio::time::sleep(delay) => {
                // 実行時刻になった
                info!(task = %task.name, "スケジュールタスクを実行");

                match execute_task(&task, &client, &tool_manager, &system_prompt).await {
                    Ok(response) => {
                        info!(task = %task.name, "タスク完了: {}", truncate(&response, 100));
                    }
                    Err(e) => {
                        error!(task = %task.name, "タスク失敗: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!(task = %task.name, "シャットダウン要求を受信");
                break;
            }
        }
    }
}

/// タスクを実行して AI の応答を取得
async fn execute_task(
    task: &ScheduleTask,
    client: &ClaudeClient,
    tool_manager: &ToolManager,
    system_prompt: &str,
) -> anyhow::Result<String> {
    use cc_core::llm::MessagesRequest;

    // ユーザーメッセージを作成
    let messages = vec![cc_core::Message::user(&task.prompt)];

    // ツール定義を取得（指定がある場合はフィルタリング）
    let tools = if task.tools.is_empty() {
        tool_manager.definitions()
    } else {
        tool_manager
            .definitions()
            .into_iter()
            .filter(|t| task.tools.contains(&t.name))
            .collect()
    };

    // リクエスト送信
    let request = MessagesRequest {
        model: client.model().to_string(),
        max_tokens: 4096,
        system: Some(system_prompt.to_string()),
        messages,
        tools: Some(tools),
    };

    let response = client.messages(request).await?;

    // テキスト応答を抽出
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

    Ok(text)
}

/// cron 文字列をパース
fn parse_cron(cron_expr: &str) -> anyhow::Result<CronSchedule> {
    // cron 形式: "分 時 日 月 曜日"
    // 例: "0 9 * * *" = 毎日 9:00
    let schedule = cron_expr.parse::<CronSchedule>()?;
    Ok(schedule)
}

/// 文字列を切り詰め
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron() {
        let result = parse_cron("0 9 * * *");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_cron_invalid() {
        let result = parse_cron("invalid");
        assert!(result.is_err());
    }
}
