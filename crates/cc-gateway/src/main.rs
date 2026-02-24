//! cc-gateway: Claude Code Gateway Main Binary
//!
//! Main entry point for the Claude Code Gateway application.
//!
//! Usage:
//!   cc-gateway           - Start server mode (HTTP API + Discord Bot + Scheduler)
//!   cc-gateway --cli     - Start interactive CLI mode
//!   cc-gateway --help    - Show help

mod cli;

use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};
use cc_mcp::McpRegistry;
use cc_schedule::{Scheduler, ScheduleConfig};
use cc_tools::register_default_tools;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Run mode
enum RunMode {
    /// Server mode (HTTP API + Discord Bot)
    Server,
    /// Interactive CLI mode
    Cli,
    /// Execute single prompt and exit (非対話モード: ワンショット実行)
    Execute(String),
    /// Execute from file and exit (非対話モード: ファイルから実行)
    File(std::path::PathBuf),
    /// Show help
    Help,
    /// Show version
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let mode = parse_args();

    match mode {
        RunMode::Help => {
            print_help();
            return Ok(());
        }
        RunMode::Version => {
            println!("cc-gateway {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => {}
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    // Load .env file (always, before TOML config)
    // This allows .env to provide API keys that TOML config references
    dotenvy::dotenv().ok();

    // Load configuration (TOML file + environment variables)
    // 環境変数は TOML 設定を上書きします
    let config = Config::load()
        .map_err(|e| anyhow::anyhow!("Config error: {}", e))?;

    tracing::info!("Starting cc-gateway...");
    tracing::info!("Model: {}", config.llm.model);

    // Create Claude client
    let claude_client = ClaudeClient::new(&config)
        .map_err(|e| anyhow::anyhow!("Failed to create LLM client: {}", e))?;

    match mode {
        RunMode::Cli => {
            // CLI mode
            tracing::info!("Running in CLI mode");
            cli::run_cli(claude_client).await
        }
        RunMode::Execute(prompt) => {
            // 非対話モード: ワンショット実行
            tracing::info!("Running in execute mode");
            cli::run_execute(claude_client, &prompt).await
        }
        RunMode::File(path) => {
            // 非対話モード: ファイルから実行
            tracing::info!("Running in file mode: {:?}", path);
            cli::run_file(claude_client, &path).await
        }
        RunMode::Server => {
            // Server mode
            run_server(config, claude_client).await
        }
        _ => Ok(()),
    }
}

/// Parse command line arguments
fn parse_args() -> RunMode {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--cli" | "-c" => return RunMode::Cli,
            "--help" | "-h" => return RunMode::Help,
            "--version" | "-v" => return RunMode::Version,
            "--execute" | "-e" => {
                if i + 1 < args.len() {
                    let prompt = args[i + 1].clone();
                    return RunMode::Execute(prompt);
                } else {
                    eprintln!("エラー: --execute には引数が必要です");
                    std::process::exit(1);
                }
            }
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    let path = std::path::PathBuf::from(&args[i + 1]);
                    return RunMode::File(path);
                } else {
                    eprintln!("エラー: --file には引数が必要です");
                    std::process::exit(1);
                }
            }
            _ => {}
        }
        i += 1;
    }

    RunMode::Server
}

/// Print help message
fn print_help() {
    println!("cc-gateway - Claude Code Gateway");
    println!();
    println!("Usage:");
    println!("  cc-gateway              Start server mode (HTTP API + Discord Bot + Scheduler)");
    println!("  cc-gateway --cli        Start interactive CLI mode");
    println!("  cc-gateway --execute PROMPT");
    println!("                          Execute single prompt and exit (非対話モード)");
    println!("  cc-gateway --file PATH  Execute prompt from file and exit (非対話モード)");
    println!("  cc-gateway --help       Show this help message");
    println!("  cc-gateway --version    Show version");
    println!();
    println!("Configuration:");
    println!("  設定は以下の優先順位で読み込まれます:");
    println!("    1. 環境変数");
    println!("    2. cc-gateway.toml 設定ファイル");
    println!("    3. デフォルト値");
    println!();
    println!("Environment Variables (環境変数は TOML 設定を上書きします):");
    println!("  LLM_API_KEY             API key (required)");
    println!("  LLM_MODEL               Model name (default: claude-sonnet-4-20250514)");
    println!("  LLM_PROVIDER            Provider: claude or openai (default: claude)");
    println!("  LLM_BASE_URL            Custom API endpoint");
    println!("  DISCORD_BOT_TOKEN       Discord bot token (optional)");
    println!("  API_PORT                HTTP API port (default: 3000)");
    println!("  MCP_ENABLED             Enable MCP integration (default: true)");
    println!("  MCP_CONFIG_PATH         Path to MCP config file");
    println!("  SCHEDULE_ENABLED        Enable scheduler (default: true)");
    println!("  SCHEDULE_CONFIG_PATH    Path to schedule.toml (default: schedule.toml)");
    println!();
    println!("Examples:");
    println!("  cc-gateway --execute \"今日の天気は？\"");
    println!("  cc-gateway -e \"2 + 2 を計算して\"");
    println!("  cc-gateway --file prompt.txt");
    println!("  cc-gateway -f ./queries/hello.txt");
}

/// Run server mode (HTTP API + Discord Bot + Scheduler)
async fn run_server(config: Config, claude_client: ClaudeClient) -> anyhow::Result<()> {
    let claude_client = Arc::new(claude_client);

    // Initialize tool manager
    let mut tool_manager = ToolManager::new();
    register_default_tools(&mut tool_manager);

    let builtin_tool_count = tool_manager.len();
    tracing::info!(
        "Registered {} built-in tools: {:?}",
        builtin_tool_count,
        tool_manager.tool_names()
    );

    // Initialize MCP integration
    let mcp_registry = if config.mcp.enabled {
        match initialize_mcp(&config, &mut tool_manager).await {
            Ok(registry) => registry,
            Err(e) => {
                tracing::warn!("MCP initialization failed: {}", e);
                None
            }
        }
    } else {
        tracing::info!("MCP integration is disabled");
        None
    };

    tracing::info!(
        "Total {} tools registered",
        tool_manager.len()
    );

    // Wrap tool_manager in Arc for sharing
    let tool_manager = Arc::new(tool_manager);

    // Create session manager
    let session_manager = SessionManager::new(&config.memory.db_path)
        .map_err(|e| anyhow::anyhow!("Failed to create session manager: {}", e))?;

    // Track running services for graceful shutdown
    let mut service_handles = Vec::new();
    let mut scheduler_handle = None;

    // Start Scheduler if enabled
    let schedule_enabled = config.scheduler.enabled;

    if schedule_enabled {
        let schedule_config = load_schedule_config();
        let enabled_count = schedule_config.enabled_tasks().len();

        if enabled_count > 0 {
            let scheduler = Scheduler::new(
                schedule_config,
                (*claude_client).clone(),
                Arc::clone(&tool_manager),
            );
            let handle = scheduler.start();
            scheduler_handle = Some(handle);
            tracing::info!("スケジューラーを開始しました ({} タスク)", enabled_count);
        } else {
            tracing::info!("スケジュールタスクがありません");
        }
    } else {
        tracing::info!("スケジューラーは無効です");
    }

    // Start Discord bot if token is configured
    if let Some(_token) = &config.discord_token {
        let discord_config = config.clone();
        let discord_client = Arc::clone(&claude_client);

        let handle = tokio::spawn(async move {
            if let Err(e) = start_discord_bot(discord_config, discord_client).await {
                tracing::error!("Discord bot error: {}", e);
            }
        });
        service_handles.push(handle);
        tracing::info!("Discord bot started");
    } else {
        tracing::info!("Discord bot disabled (no token configured)");
    }

    // Start HTTP API server
    let api_port = config.api.port;
    let api_config = config.clone();
    let api_client = Arc::clone(&claude_client);
    let api_tool_manager = Arc::clone(&tool_manager);

    let handle = tokio::spawn(async move {
        if let Err(e) = cc_api::start_server(
            api_port,
            api_config,
            (*api_client).clone(),
            session_manager,
            api_tool_manager,
        ).await {
            tracing::error!("HTTP API error: {}", e);
        }
    });
    service_handles.push(handle);
    tracing::info!("HTTP API server started on port {}", api_port);

    tracing::info!("cc-gateway initialized successfully");
    tracing::info!("Press Ctrl+C to exit");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");

    // Stop scheduler
    if let Some(handle) = scheduler_handle {
        handle.stop().await;
    }

    // Abort all services
    for handle in service_handles {
        handle.abort();
    }

    // Gracefully shutdown MCP clients
    if let Some(registry) = mcp_registry {
        if let Err(e) = registry.shutdown().await {
            tracing::warn!("Error during MCP shutdown: {}", e);
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

/// Load schedule configuration
fn load_schedule_config() -> ScheduleConfig {
    // Check for custom config path
    if let Ok(path) = std::env::var("SCHEDULE_CONFIG_PATH") {
        tracing::info!("Loading schedule config from: {}", path);
        match ScheduleConfig::from_file(&path) {
            Ok(config) => return config,
            Err(e) => tracing::warn!("Failed to load schedule config from {}: {}", path, e),
        }
    }

    // Try default paths
    ScheduleConfig::load_default().unwrap_or_default()
}

/// Start Discord bot
async fn start_discord_bot(config: Config, claude_client: Arc<ClaudeClient>) -> anyhow::Result<()> {
    use cc_discord::DiscordBot;

    let bot = DiscordBot::with_client(config, claude_client);
    bot.start().await
        .map_err(|e| anyhow::anyhow!("Discord bot error: {}", e))
}

/// Initialize MCP tools from configuration
async fn initialize_mcp(
    config: &Config,
    tool_manager: &mut ToolManager,
) -> anyhow::Result<Option<McpRegistry>> {
    use cc_mcp::McpConfig;

    let mcp_config = match &config.mcp.config_path {
        Some(path) => {
            tracing::info!("Loading MCP configuration from: {}", path);
            McpConfig::from_json_file(path)
                .map_err(|e| anyhow::anyhow!("Failed to load MCP config: {}", e))?
        }
        None => {
            // Try default path
            let default_path = "mcp.json";
            if std::path::Path::new(default_path).exists() {
                tracing::info!("Loading MCP configuration from default path: {}", default_path);
                McpConfig::from_json_file(default_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load MCP config: {}", e))?
            } else {
                tracing::info!("No MCP configuration file found, skipping MCP initialization");
                return Ok(None);
            }
        }
    };

    McpRegistry::initialize(&mcp_config, tool_manager)
        .await
        .map_err(|e| anyhow::anyhow!("MCP registry initialization failed: {}", e))
}
