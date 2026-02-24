//! CLI (Command Line Interface) mode
//!
//! Provides an interactive REPL for OpenClaw-like experience.
//! Also supports non-interactive execute mode for one-shot execution.

use cc_core::{ClaudeClient, Message, MessageContent, ToolManager, ToolResult};
use cc_core::llm::{MessagesRequest, ToolDefinition};
use cc_tools::register_default_tools;
use nu_ansi_term::{Color, Style};
use reedline::{
    ColumnarMenu, Completer, DefaultHinter, Emacs, KeyCode, KeyModifiers,
    Keybindings, MenuBuilder, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal, Suggestion,
};
use serde_json::Value as JsonValue;
use std::path::Path;
use tracing::info;

/// Available commands for autocomplete display
const COMMANDS: &[(&str, &str)] = &[
    ("/help", "ヘルプを表示"),
    ("/exit", "プログラムを終了"),
    ("/quit", "プログラムを終了"),
    ("/clear", "会話履歴をクリア"),
    ("/history", "会話履歴を表示"),
];

/// Command completer for reedline
#[derive(Clone)]
pub struct CommandCompleter {
    commands: Vec<(&'static str, &'static str)>,
}

impl CommandCompleter {
    pub fn new() -> Self {
        Self {
            commands: COMMANDS.to_vec(),
        }
    }
}

impl Default for CommandCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // 行頭が / で始まる場合は常に候補を表示
        // pos != line.len() のチェックを削除して、編集中でも候補を表示
        if !line.starts_with('/') {
            return Vec::new();
        }

        self.commands
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(line))
            .map(|(cmd, desc)| Suggestion {
                value: cmd.to_string(),
                description: Some(desc.to_string()),
                extra: None,
                span: reedline::Span::new(0, pos),
                append_whitespace: true,
                style: None,
            })
            .collect()
    }
}

/// Custom prompt with colored styling
struct ColoredPrompt {
    style: Style,
}

impl ColoredPrompt {
    fn new() -> Self {
        Self {
            style: Color::Cyan.bold(),
        }
    }
}

impl Prompt for ColoredPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(self.style.paint("> ").to_string())
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("")
    }
}

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

    // Setup keybindings
    let mut keybindings = default_keybindings();

    // Trigger completion on '/' key
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Char('/'),
        ReedlineEvent::Edit(vec![reedline::EditCommand::Complete]),
    );

    // Setup menu - with_only_buffer_difference(false) makes menu show even without buffer changes
    let menu = Box::new(
        ColumnarMenu::default()
            .with_name("command_menu")
            .with_columns(1)
            .with_column_width(Some(40))
            .with_only_buffer_difference(false),
    );

    // Setup hinter
    let hinter = DefaultHinter::default().with_style(Style::new().dimmed());

    // Create line editor
    let mut line_editor = Reedline::create()
        .with_completer(Box::new(CommandCompleter::new()))
        .with_menu(ReedlineMenu::EngineCompleter(menu))
        .with_hinter(Box::new(hinter))
        .with_edit_mode(Box::new(Emacs::new(keybindings)));

    let prompt = ColoredPrompt::new();

    // Conversation history
    let mut messages: Vec<cc_core::Message> = Vec::new();

    loop {
        let signal = line_editor.read_line(&prompt);

        match signal {
            Ok(Signal::Success(line)) => {
                let input = line.trim();

                // Handle empty input
                if input.is_empty() {
                    continue;
                }

                // Handle special commands
                if handle_command(input, &mut messages) {
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
                        eprintln!("\n❌ エラー: {}\n", e);
                    }
                }
            }
            Ok(Signal::CtrlC) => {
                println!("^C");
                continue;
            }
            Ok(Signal::CtrlD) => {
                println!("\n👋 さようなら！\n");
                break;
            }
            Err(err) => {
                eprintln!("\n❌ エラー: {}\n", err);
                break;
            }
        }
    }

    Ok(())
}

/// Default keybindings for reedline
fn default_keybindings() -> Keybindings {
    let mut keybindings = Keybindings::new();
    // Tab key triggers completion
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Edit(vec![reedline::EditCommand::Complete]),
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Enter,
        ReedlineEvent::Submit,
    );
    // Esc key clears/closes menus
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Esc,
        ReedlineEvent::Esc,
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('c'),
        ReedlineEvent::CtrlC,
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('d'),
        ReedlineEvent::CtrlD,
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Up,
        ReedlineEvent::Up,
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Down,
        ReedlineEvent::Down,
    );
    keybindings
}

/// Handle special commands (/, /exit, /clear, /help)
fn handle_command(input: &str, messages: &mut Vec<cc_core::Message>) -> bool {
    let lower = input.to_lowercase();

    match lower.as_str() {
        "/exit" | "/quit" | "/q" => {
            println!("\n👋 さようなら！\n");
            std::process::exit(0);
        }
        "/clear" => {
            messages.clear();
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
            thinking: None,
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
    println!("║  / を入力するとコマンド候補が表示されます                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}

/// Print help message
fn print_help() {
    println!();
    println!("📖 利用可能なコマンド:");
    for (cmd, desc) in COMMANDS {
        println!("  {} - {}", cmd, desc);
    }
    println!();
    println!("💡 ヒント: / から入力するとコマンド候補が表示されます");
    println!("💡 矢印キー(↑/↓)で候補を選択、Enterで確定できます");
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

// ============================================================================
// 非対話モード (Non-interactive mode)
// ============================================================================

/// システムプロンプト（非対話モード用）
const SYSTEM_PROMPT: &str = "あなたはツールにアクセスできる便利な AI アシスタントです。\
    ユーザーと同じ言語で応答してください。\
    必要に応じてツールを使用してユーザーを支援してください。";

/// 最大反復回数（非対話モード用）
const MAX_ITERATIONS: usize = 10;

/// 非対話モード: プロンプトを直接実行して終了
///
/// # 使用例
/// ```bash
/// cc-gateway --execute "今日の天気は？"
/// cc-gateway -e "2 + 2 を計算して"
/// ```
pub async fn run_execute(client: ClaudeClient, prompt: &str) -> anyhow::Result<()> {
    // プロンプトが空の場合はエラー
    let prompt = prompt.trim();
    if prompt.is_empty() {
        eprintln!("エラー: プロンプトが空です");
        std::process::exit(1);
    }

    // ツールマネージャーを初期化
    let mut tool_manager = ToolManager::new();
    register_default_tools(&mut tool_manager);

    info!("Starting execute mode with {} tools", tool_manager.len());

    // メッセージを構築
    let mut messages: Vec<Message> = vec![Message::user(prompt)];

    // Agent turn を実行
    match run_agent_turn(
        &client,
        &mut messages,
        SYSTEM_PROMPT,
        &tool_manager,
        MAX_ITERATIONS,
    )
    .await
    {
        Ok(response) => {
            // レスポンスを出力
            println!("{}", response);
            Ok(())
        }
        Err(e) => {
            eprintln!("エラー: {}", e);
            std::process::exit(1);
        }
    }
}

/// 非対話モード: ファイルからプロンプトを読み込んで実行
///
/// # 使用例
/// ```bash
/// cc-gateway --file prompt.txt
/// cc-gateway -f ./queries/hello.txt
/// ```
pub async fn run_file(client: ClaudeClient, path: &Path) -> anyhow::Result<()> {
    // ファイルの存在チェック
    if !path.exists() {
        eprintln!("エラー: ファイルが存在しません: {}", path.display());
        std::process::exit(1);
    }

    // ファイルからプロンプトを読み込み
    let prompt = tokio::fs::read_to_string(path).await;
    let prompt = match prompt {
        Ok(content) => content,
        Err(e) => {
            eprintln!("エラー: ファイルの読み込みに失敗しました: {}", e);
            std::process::exit(1);
        }
    };

    let prompt = prompt.trim();
    if prompt.is_empty() {
        eprintln!("エラー: ファイルの内容が空です: {}", path.display());
        std::process::exit(1);
    }

    info!("Executing prompt from file: {}", path.display());

    // execute モードと同じ処理を実行
    run_execute(client, prompt).await
}
