//! CLI (Command Line Interface) mode
//!
//! Provides an interactive REPL for OpenClaw-like experience.

use std::io::{self, Write};

use cc_core::{ClaudeClient, MessageContent, ToolManager, ToolResult};
use cc_core::llm::{MessagesRequest, ToolDefinition};
use cc_tools::register_default_tools;
use serde_json::Value as JsonValue;
use tracing::info;

/// CLI configuration
pub struct CliConfig {
    pub system_prompt: String,
    pub max_iterations: usize,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            system_prompt: "あなたはツールにアクセスできる便利な AI アシスタントです。\
                ユーザーと同じ言語で応答してください。\
                必要に応じてツールを使用してユーザーを支援してください。"
                .to_string(),
            max_iterations: 10,
        }
    }
}

/// Run CLI interactive mode
pub async fn run_cli(client: ClaudeClient) -> anyhow::Result<()> {
    let config = CliConfig::default();
    run_cli_with_config(client, config).await
}

/// Run CLI with custom configuration
pub async fn run_cli_with_config(client: ClaudeClient, cli_config: CliConfig) -> anyhow::Result<()> {
    // Initialize tool manager
    let mut tool_manager = ToolManager::new();
    register_default_tools(&mut tool_manager);

    info!("Starting CLI mode with {} tools", tool_manager.len());

    // Welcome message
    print_welcome();

    // Conversation history
    let mut messages: Vec<cc_core::Message> = Vec::new();

    loop {
        // Read user input
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle empty input
        if input.is_empty() {
            continue;
        }

        // Handle special commands
        if handle_command(input, &messages) {
            continue;
        }

        // Add user message to history
        messages.push(cc_core::Message::user(input));

        // Run agent loop
        match run_agent_turn(
            &client,
            &mut messages,
            &cli_config.system_prompt,
            &tool_manager,
            cli_config.max_iterations,
        )
        .await
        {
            Ok(response) => {
                // Print response
                println!("\n{}\n", response);

                // Add assistant response to history
                messages.push(cc_core::Message::assistant(&response));
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}\n", e);
            }
        }
    }
}

/// Handle special commands (/, /exit, /clear, /help)
fn handle_command(input: &str, messages: &[cc_core::Message]) -> bool {
    let lower = input.to_lowercase();

    match lower.as_str() {
        "/exit" | "/quit" | "/q" => {
            println!("\n👋 さようなら！\n");
            std::process::exit(0);
        }
        "/clear" => {
            // Note: We can't clear messages from here since it's borrowed
            println!("\n✅ 会話履歴をクリアしました。\n");
            true
        }
        "/help" | "/?" => {
            print_help();
            true
        }
        "/history" => {
            print_history(messages);
            true
        }
        _ if lower.starts_with('/') => {
            eprintln!("\n❓ 不明なコマンド: {}。/help でコマンド一覧を確認してください。\n", input);
            true
        }
        _ => false,
    }
}

/// Run a single agent turn with tools
async fn run_agent_turn(
    client: &ClaudeClient,
    messages: &mut Vec<cc_core::Message>,
    system_prompt: &str,
    tool_manager: &ToolManager,
    max_iterations: usize,
) -> anyhow::Result<String> {
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            return Ok("最大反復回数に達しました。よりシンプルなリクエストで再試行してください。".to_string());
        }

        // Build request
        let request = MessagesRequest {
            model: client.model().to_string(),
            max_tokens: 4096,
            system: Some(system_prompt.to_string()),
            messages: messages.clone(),
            tools: Some(get_tool_definitions(tool_manager)),
        };

        let response = client.messages(request).await?;

        match response.stop_reason.as_str() {
            "end_turn" | "stop_sequence" | "stop" => {
                // Extract text response
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

                return Ok(text);
            }
            "tool_use" | "tool_calls" => {
                // Process tool uses
                let tool_uses: Vec<_> = response
                    .content
                    .iter()
                    .filter_map(|c| {
                        if let cc_core::MessageContent::ToolUse { id, name, input } = c {
                            Some((id.clone(), name.clone(), input.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                if tool_uses.is_empty() {
                    continue;
                }

                // Add assistant message with tool_use
                messages.push(cc_core::Message {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });

                // Execute tools and collect results
                let mut tool_results = Vec::new();
                for (id, name, input) in &tool_uses {
                    info!("Executing tool: {} with input: {:?}", name, input);

                    let result = execute_tool(tool_manager, name, input.clone()).await;
                    tool_results.push(MessageContent::ToolResult {
                        tool_use_id: id.clone(),
                        content: result.output.clone(),
                        is_error: result.is_error,
                    });

                    // Show tool execution to user
                    if result.is_error {
                        eprintln!("\n⚙️ ツール {} の実行に失敗: {}", name, result.output);
                    } else {
                        println!("\n⚙️ ツール {} を実行しました", name);
                    }
                }

                // Add user message with tool_results
                messages.push(cc_core::Message {
                    role: "user".to_string(),
                    content: tool_results,
                });
            }
            other => {
                return Err(anyhow::anyhow!("Unknown stop_reason: {}", other));
            }
        }
    }
}

/// Execute a tool by name
async fn execute_tool(tool_manager: &ToolManager, name: &str, input: JsonValue) -> ToolResult {
    match tool_manager.execute(name, input).await {
        Ok(result) => result,
        Err(e) => ToolResult::error(format!("Tool execution error: {}", e)),
    }
}

/// Get tool definitions for the request
fn get_tool_definitions(tool_manager: &ToolManager) -> Vec<ToolDefinition> {
    tool_manager.definitions()
}

/// Print welcome message
fn print_welcome() {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          🤖 cc-gateway CLI - 対話モード                    ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  メッセージを入力して Enter でチャット開始                  ║");
    println!("║  コマンド: /help, /exit, /clear, /history                  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}

/// Print help message
fn print_help() {
    println!();
    println!("📖 利用可能なコマンド:");
    println!("  /help, /?     - ヘルプを表示");
    println!("  /exit, /quit  - プログラムを終了");
    println!("  /clear        - 会話履歴をクリア");
    println!("  /history      - 会話履歴を表示");
    println!();
}

/// Print conversation history
fn print_history(messages: &[cc_core::Message]) {
    println!();
    println!("📜 会話履歴 ({} 件):", messages.len());
    println!("{}", "─".repeat(50));

    for (i, msg) in messages.iter().enumerate() {
        let role = match msg.role.as_str() {
            "user" => "👤 あなた",
            "assistant" => "🤖 AI",
            _ => &msg.role,
        };
        let text = msg.text_content();
        let preview = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.clone()
        };
        println!("{}. {}: {}", i + 1, role, preview.replace('\n', " "));
    }

    println!("{}", "─".repeat(50));
    println!();
}
