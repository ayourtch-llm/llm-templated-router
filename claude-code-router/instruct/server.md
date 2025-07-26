Create an HTTP server module for the claude-code-router that integrates with config and routing modules.

Requirements:
1. Import required modules:
   - Config from crate::config module
   - Router, RouterRequest, Message, Tool from crate::router module
   - ProviderClient from crate::provider module
   - hyper components for HTTP handling
   - serde_json for JSON parsing

2. Create a Server struct with fields:
   - config: Config
   - router: Router  
   - provider_client: ProviderClient
   - shutdown_tx: Option<oneshot::Sender<()>>

3. Implement these methods:
   - new(config: Config) -> Server (initialize router and provider_client)
   - start(&mut self) -> Result<(), Box<dyn std::error::Error>>
   - stop(&mut self) -> Result<(), Box<dyn std::error::Error>>

4. HTTP endpoint handling:
   - GET "/" and "/health" -> 200 OK with "OK" body (health checks)
   - POST "/v1/messages" -> Claude API endpoint with full request forwarding
   - Other routes -> 404 Not Found

5. Claude API request processing:
   - Parse JSON request body 
   - Extract: model, messages, system, tools, thinking fields
   - Convert to RouterRequest struct
   - Call router.route_request() to determine target provider/model
   - Use provider_client.send_request() to forward to actual LLM provider
   - Return the provider's response directly to client

6. Response formats:
   - Health checks: plain text "OK"
   - Successful forwarding: Return provider's JSON response as-is
   - Routing errors: JSON error messages with appropriate HTTP status codes
   - Provider errors: Forward provider error responses

7. Authentication middleware:
   - Check Authorization header or x-api-key header  
   - Validate against config.apikey if set
   - Skip auth for health endpoints
   - Return 401 for invalid/missing API keys

8. Error handling:
   - Graceful JSON parsing error handling
   - Proper HTTP status codes (400, 401, 404, 500)
   - Log errors and routing decisions

9. Use hyper 0.14 with proper async/await patterns and graceful shutdown

This creates a Claude API-compatible router that processes requests and shows routing decisions.