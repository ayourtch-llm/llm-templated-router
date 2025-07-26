Create a CLI for the claude-code-router that integrates with server and config modules.

Requirements:
1. Add imports for config and server modules:
   - use claude_code_router::config::{load_config, Config}
   - use claude_code_router::server::Server

2. Update main function to be async: 
   - Use #[tokio::main] attribute
   - Change fn main() to async fn main() -> Result<(), Box<dyn std::error::Error>>

3. Commands to implement:
   - start: Load config, create server, start HTTP service
   - stop: Print message about stop functionality (process management TBD)
   - status: Print basic status message (process checking TBD)
   - help: Show help text

4. Start command implementation:
   - Load config using load_config()
   - Create Server::new(config)
   - Call server.start().await
   - Handle errors gracefully with user-friendly messages

5. Use clap for argument parsing with same command structure

6. Add proper error handling and startup messages with emojis

7. Keep the process running when server starts

This integrates the CLI with the server module for a working router service.