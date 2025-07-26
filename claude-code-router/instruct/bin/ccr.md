Create a CLI for the claude-code-router that integrates with server and config modules.

Requirements:
1. Add imports for config and server modules:
   - use claude_code_router::config::{load_config, Config}
   - use claude_code_router::server::Server
   - use std::process::Command for executing claude CLI

2. Update main function to be async: 
   - Use #[tokio::main] attribute
   - Change fn main() to async fn main() -> Result<(), Box<dyn std::error::Error>>

3. Commands to implement:
   - start: Load config, create server, start HTTP service
   - stop: Print message about stop functionality (process management TBD)
   - status: Print basic status message (process checking TBD)
   - code <args>: Execute Claude Code CLI through the router
   - help: Show help text

4. Start command implementation:
   - Load config using load_config()
   - Create Server::new(config)
   - Call server.start().await
   - Handle errors gracefully with user-friendly messages

5. Code command implementation:
   - Check if service is running (basic check for now)
   - Set environment variables:
     - ANTHROPIC_BASE_URL=http://127.0.0.1:8080 (or config host)
     - ANTHROPIC_API_KEY=config.apikey (if set)
     - API_TIMEOUT_MS=600000
   - Execute "claude" command with remaining arguments
   - Pass through exit code

6. Use clap for argument parsing:
   - Use #[clap(trailing_var_arg = true)] for code command args
   - Handle remaining arguments after "code" subcommand

7. Add proper error handling and startup messages with emojis

8. Keep the process running when server starts

This integrates the CLI with the server module and provides Claude Code execution through the router.