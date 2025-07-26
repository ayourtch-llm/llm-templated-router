Create a basic HTTP server module for the claude-code-router that integrates with existing config module.

Requirements:
1. Import the Config struct from crate::config module (not define a new one)

2. Create a Server struct that can:
   - Start HTTP server on specified host and port from Config.host field
   - Handle shutdown gracefully using tokio oneshot channels
   - Accept configuration from existing Config struct

3. Use hyper 0.14 for HTTP server with tokio runtime

4. Implement these methods:
   - new(config: Config) -> Server
   - start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
   - stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>

5. The server should:
   - Parse host from Config.host field (format "host:port")
   - Handle basic HTTP requests (GET, POST) 
   - Return 200 OK with "OK" body for health checks on "/"
   - Return 404 for other routes
   - Print startup/shutdown messages with emojis

6. Server struct fields:
   - config: Config
   - shutdown_tx: Option<oneshot::Sender<()>>

7. Include async router function that handles:
   - GET / -> 200 OK
   - POST / -> 200 OK  
   - Other routes -> 404 Not Found

8. Include basic tests using tokio::test

9. Use proper error handling with Box<dyn std::error::Error + Send + Sync>

10. Server must be non-blocking and use graceful shutdown pattern

This integrates with the existing config.rs module and serves as foundation for routing logic.