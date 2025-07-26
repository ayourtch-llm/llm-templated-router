use clap::{Parser, Subcommand};
use claude_code_router::config::load_config;
use claude_code_router::server::Server;
use std::process::{Command, Stdio};
use std::fs;
use std::env;
use reqwest;
use tokio::time::{sleep, Duration};

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
            
            let pid = std::process::id();
            if let Err(e) = fs::write("/tmp/ccr.pid", pid.to_string()) {
                eprintln!("❌ Failed to write PID file: {}", e);
                return Err(e.into());
            }
            
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
            
            // Clean up PID file
            let _ = fs::remove_file("/tmp/ccr.pid");
        }
        Commands::Stop => {
            println!("⏹️  Stopping claude-code-router service...");
            
            match fs::read_to_string("/tmp/ccr.pid") {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<u32>() {
                        Ok(pid) => {
                            let status = Command::new("kill")
                                .arg(pid.to_string())
                                .status();
                            
                            match status {
                                Ok(status) if status.success() => {
                                    let _ = fs::remove_file("/tmp/ccr.pid");
                                    println!("✅ Service stopped successfully");
                                }
                                _ => {
                                    println!("❌ Failed to stop process or process not found");
                                }
                            }
                        }
                        Err(_) => {
                            println!("❌ Invalid PID in file");
                        }
                    }
                }
                Err(_) => {
                    println!("ℹ️  No running service found");
                }
            }
        }
        Commands::Status => {
            println!("📊 Checking service status...");
            
            match fs::read_to_string("/tmp/ccr.pid") {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<u32>() {
                        Ok(pid) => {
                            let client = reqwest::Client::new();
                            let config = match load_config() {
                                Ok(config) => config,
                                Err(_) => {
                                    println!("❌ Failed to load configuration for health check");
                                    return Ok(());
                                }
                            };
                            let host = config.host.as_deref().unwrap_or("127.0.0.1:8080");
                            let health_url = format!("http://{}/health", host);
                            
                            match client.get(&health_url).send().await {
                                Ok(_) => {
                                    println!("✅ Service is running (PID: {})", pid);
                                }
                                Err(_) => {
                                    println!("⚠️  Service is not responding (PID: {})", pid);
                                }
                            }
                        }
                        Err(_) => {
                            println!("❌ Invalid PID in file");
                        }
                    }
                }
                Err(_) => {
                    println!("ℹ️  No running service found");
                }
            }
        }
        Commands::Code { args } => {
            println!("🔍 Checking service status...");
            
            // Check if service is running
            let config = match load_config() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("❌ Failed to load configuration: {}", e);
                    std::process::exit(1);
                }
            };
            
            let host = config.host.as_deref().unwrap_or("127.0.0.1:8080");
            let health_url = format!("http://{}/health", host);
            
            let client = reqwest::Client::new();
            let service_running = client.get(&health_url).send().await.is_ok();
            
            if !service_running {
                println!("⚠️  Service not running, starting in background...");
                
                // Start server in background
                let log_file = "/tmp/ccr.log";
                env::set_var("RUST_LOG", "debug");
                
                let mut cmd = Command::new("nohup");
                cmd.arg(std::env::current_exe().unwrap())
                    .arg("start")
                    .stdout(Stdio::from(std::fs::File::create(log_file).expect("Failed to create log file")))
                    .stderr(Stdio::from(std::fs::File::create(log_file).expect("Failed to create log file")))
                    .spawn()
                    .expect("Failed to start server in background");
                
                println!("⏳ Waiting 3 seconds for server to start...");
                sleep(Duration::from_secs(3)).await;
                
                // Verify server started
                let service_running = client.get(&health_url).send().await.is_ok();
                if !service_running {
                    eprintln!("❌ Failed to start service in background");
                    std::process::exit(1);
                }
                
                println!("✅ Service started successfully in background");
            }
            
            // Set environment variables
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