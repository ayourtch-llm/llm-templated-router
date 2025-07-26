use clap::{Parser, Subcommand};
use claude_code_router::config::load_config;
use claude_code_router::server::Server;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    }
    
    Ok(())
}