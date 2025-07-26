Create a basic HTTP server module for the claude-code-router.

Requirements:
1. Create a Server struct that can:
   - Start HTTP server on specified host and port
   - Handle shutdown gracefully
   - Accept configuration from Config struct

2. Use tokio for async runtime and a simple HTTP server library (like hyper or warp)

3. Implement these methods:
   - new(config: Config) -> Server
   - start(&self) -> Result<(), Box<dyn std::error::Error>>
   - stop(&self) -> Result<(), Box<dyn std::error::Error>>

4. The server should:
   - Bind to host:port from config
   - Handle basic HTTP requests (GET, POST)
   - Return 200 OK for health checks on "/"
   - Print startup/shutdown messages

5. Add required dependencies to Cargo.toml:
   - hyper or warp for HTTP server
   - tower for middleware (if using hyper)

6. The server must be non-blocking and handle concurrent requests

7. Include proper error handling and logging

This is a foundational server that will later be extended with routing logic.