Create a provider module for sending requests to LLM providers and returning responses.

Requirements:

1. Add HTTP client dependency to Cargo.toml:
   - reqwest = { version = "0.11", features = ["json"] }

2. Create a ProviderClient struct with methods:
   - new() -> ProviderClient  
   - send_request(&self, provider_route: &str, request: &RouterRequest, config: &Config) -> Result<serde_json::Value, Box<dyn std::error::Error>>

3. Route parsing logic:
   - Parse "provider,model" format (e.g., "groq,moonshotai/kimi-k2-instruct")
   - Extract provider name and model name
   - Find matching provider in config.providers by name
   - Return error if provider not found

4. Request transformation:
   - Convert RouterRequest to provider-specific format
   - Handle different provider API formats (OpenAI-compatible vs native)
   - Set correct headers (Authorization, Content-Type, User-Agent)
   - Use provider's api_base_url and api_key from config

5. HTTP request handling:
   - Use reqwest for async HTTP requests
   - Set appropriate timeouts (30s default)
   - Handle different HTTP methods (POST for most providers)
   - Stream response handling for large responses

6. Response processing:
   - Parse JSON response from provider
   - Handle error responses (4xx, 5xx status codes)
   - Extract and format response data
   - Return raw JSON for flexibility

7. Error handling:
   - Network errors (connection, timeout)
   - HTTP errors (status codes)
   - JSON parsing errors
   - Provider-specific error formats
   - Log errors with provider context

8. Provider-specific handling:
   - OpenAI-compatible endpoints (most providers)
   - Special cases for specific providers if needed
   - Header requirements per provider
   - Model name formatting

9. Request format for OpenAI-compatible providers:
   - POST to {api_base_url}
   - Headers: Authorization: Bearer {api_key}, Content-Type: application/json
   - Body: {"model": model_name, "messages": [...], "system": "...", "tools": [...]}

This creates the HTTP client for forwarding routed requests to actual LLM providers.