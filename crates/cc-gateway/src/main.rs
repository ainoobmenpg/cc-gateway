//! cc-gateway: Claude Code Gateway Main Binary
//!
//! Main entry point for the Claude Code Gateway application.
//!
//! Usage:
//!   cc-gateway           - Start server mode (HTTP API + Discord Bot)
//!   cc-gateway --cli     - Start interactive CLI mode
//!   cc-gateway --help    - Show help

mod cli;

use cc_core::{ClaudeClient, Config, SessionManager, ToolManager};
use cc_mcp::McpRegistry;
use cc_tools::register_default_tools;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Run mode
enum RunMode {
    /// Server mode (HTTP API + Discord Bot)
    Server,
    /// Interactive CLI mode
    Cli,
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

    // Load .env file
    dotenvy::dotenv().ok();

    // Load configuration from environment
    let config = Config::from_env()
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

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--cli" | "-c" => return RunMode::Cli,
            "--help" | "-h" => return RunMode::Help,
            "--version" | "-v" => return RunMode::Version,
            _ => {}
        }
    }

    RunMode::Server
}

/// Print help message
fn print_help() {
    println!("cc-gateway - Claude Code Gateway");
    println!();
    println!("Usage:");
    println!("  cc-gateway           Start server mode (HTTP API + Discord Bot)");
    println!("  cc-gateway --cli     Start interactive CLI mode");
    println!("  cc-gateway --help    Show this help message");
    println!("  cc-gateway --version Show version");
    println!();
    println!("Environment Variables:");
    println!("  LLM_API_KEY          API key (required)");
    println!("  LLM_MODEL            Model name (default: claude-sonnet-4-20250514)");
    println!("  LLM_PROVIDER         Provider: claude or openai (default: claude)");
    println!("  LLM_BASE_URL         Custom API endpoint");
    println!("  DISCORD_BOT_TOKEN    Discord bot token (optional)");
    println!("  API_PORT             HTTP API port (default: 3000)");
    println!("  MCP_ENABLED          Enable MCP integration (default: true)");
    println!("  MCP_CONFIG_PATH      Path to MCP config file");
}

/// Run server mode (HTTP API + Discord Bot)
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

    // Create session manager
    let session_manager = SessionManager::new(&config.memory.db_path)
        .map_err(|e| anyhow::anyhow!("Failed to create session manager: {}", e))?;

    // Track running services for graceful shutdown
    let mut service_handles = Vec::new();

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

    let handle = tokio::spawn(async move {
        if let Err(e) = cc_api::start_server(api_port, api_config, (*api_client).clone(), session_manager).await {
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

/// Start Discord bot
async fn start_discord_bot(config: Config, claude_client: Arc<ClaudeClient>) -> anyhow::Result<()> {
    use cc_discord::DiscordBot;

    let bot = DiscordBot::with_client(config, claude_client);
    bot.start().await
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
