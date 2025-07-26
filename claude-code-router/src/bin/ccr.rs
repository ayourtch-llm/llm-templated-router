use clap::{Parser, Subcommand};

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
    /// Show help text
    Help,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            println!("Starting claude-code-router service...");
        }
        Commands::Stop => {
            println!("Stopping claude-code-router service...");
        }
        Commands::Status => {
            println!("Service status: running");
        }
        Commands::Help => {
            println!("claude-code-router CLI");
            println!("USAGE:");
            println!("  claude-code-router <COMMAND>");
            println!();
            println!("COMMANDS:");
            println!("  start   Start the router service");
            println!("  stop    Stop the router service");
            println!("  status  Show service status");
            println!("  help    Show help text");
        }
    }
}