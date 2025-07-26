use clap::{Parser, Subcommand};
use claude_code_router::config::load_config;
use claude_code_router::server::Server;
use std::process::{Command, Stdio};
use std::env;

#[derive(Parser)]
#[command(name = "claude-code-router")]
#[command(about = "A CLI for managing the claude-code-router service", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the router service
    Start,
    /// Stop the router service
    Stop,
    /// Show service status
    Status,
    /// Execute Claude Code CLI through the router
    Code {
        #[clap(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            println!("🚀 Starting claude-code-router service...");
            
            let config = load_config().map_err(|e| {
                eprintln!("❌ Failed to load configuration: {}", e);
                e
            })?;
            
            let mut server = Server::new(config);
            
            println!("✅ Configuration loaded successfully");
            println!("🌐 Starting HTTP service...");
            
            server.start().await.map_err(|e| {
                eprintln!("❌ Failed to start server: {}", e);
                e
            })?;
            
            println!("✅ claude-code-router service started successfully!");
            
            // Keep the process running
            tokio::signal::ctrl_c().await?;
            println!("\n👋 Shutting down gracefully...");
        }
        Commands::Stop => {
            println!("⏹️  Stopping claude-code-router service...");
            println!("ℹ️  Process management functionality is TBD");
        }
        Commands::Status => {
            println!("📊 Service status checking functionality is TBD");
            println!("Current status: configuration available but process monitoring not implemented");
        }
        Commands::Code { args } => {
            println!("🔍 Checking service status...");
            // Basic check - in a real implementation, this would check if the service is running
            println!("⚠️  Service status check not implemented - assuming service is running");
            
            // Load config to get settings
            let config = load_config().map_err(|e| {
                eprintln!("❌ Failed to load configuration: {}", e);
                std::process::exit(1);
            }).unwrap();
            
            // Set environment variables
            let host = config.host.as_deref().unwrap_or("127.0.0.1:8080");
            let base_url = format!("http://{}", host);
            env::set_var("ANTHROPIC_BASE_URL", &base_url);
            if let Some(api_key) = &config.apikey {
                env::set_var("ANTHROPIC_API_KEY", api_key);
            }
            env::set_var("API_TIMEOUT_MS", "600000");
            
            println!("🚀 Executing Claude Code CLI with args: {:?}", args);
            
            let status = Command::new("claude")
                .args(args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("Failed to execute claude command");
            
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    
    Ok(())
}