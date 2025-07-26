Create a provider module for sending requests to LLM providers and returning responses.

Requirements:

1. Add HTTP client dependency to Cargo.toml:
   - reqwest = { version = "0.11", features = ["json"] }

2. Create a ProviderClient struct with methods:
   - new() -> ProviderClient (with 30s timeout and user-agent "router/0.1")
   - send_request(&self, provider_route: &str, request: &RouterRequest, config: &Config) -> Result<serde_json::Value, Box<dyn std::error::Error>>
   - send_claude_request(&self, provider_route: &str, claude_req: &ClaudeRequest, config: &Config, transformed_messages: Vec<serde_json::Value>, transformed_tools: Option<Vec<serde_json::Value>>) -> Result<serde_json::Value, Box<dyn std::error::Error>>
   - apply_transformers(&self, body: &mut serde_json::Value, claude_req: &ClaudeRequest, provider: &Provider) -> Result<(), Box<dyn std::error::Error>>
   - apply_transformer_use(&self, body: &mut serde_json::Value, claude_req: &ClaudeRequest, transformer_use: &TransformerUse) -> Result<(), Box<dyn std::error::Error>>
   - convert_openai_to_claude_format(&self, openai_response: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>>

3. Route parsing logic:
   - Parse "provider,model" format (e.g., "groq,moonshotai/kimi-k2-instruct")
   - Extract provider name and model name
   - Find matching provider in config.providers by name
   - Return error if provider not found

4. Request transformation:
   - Use pre-transformed messages and tools from MessageTransformer (passed as parameters)
   - Create OpenAI-compatible request body using transformed_messages and transformed_tools
   - Build base request body: {"model": model, "messages": transformed_messages}
   - Add optional fields: max_tokens, temperature from claude_req
   - ALWAYS set "stream": false (streaming not yet supported)
   - Add tools if transformed_tools provided
   - Apply provider-specific transformers based on config (openrouter, gemini, maxtoken, etc.)
   - Set correct headers (Authorization Bearer token, Content-Type application/json)
   - Use provider's api_base_url and api_key from config
   - Handle URL construction: append "/chat/completions" if not already present

5. HTTP request handling:
   - Use reqwest for async HTTP requests
   - Set appropriate timeouts (30s default)
   - Handle different HTTP methods (POST for most providers)
   - Stream response handling for large responses

6. Response transformation and processing:
   - Parse JSON response from provider
   - Convert OpenAI format responses to Claude API format for compatibility
   - Transform OpenAI structure: {"choices": [{"message": {"content": "text"}}]} 
   - Into Claude structure: {"type": "message", "role": "assistant", "content": [{"type": "text", "text": "content"}]}
   - Map finish_reason: "stop" -> "end_turn", "length" -> "max_tokens", "tool_calls" -> "tool_use"
   - Transform tool_calls into tool_use content blocks with proper Claude format
   - Extract usage stats: prompt_tokens -> input_tokens, completion_tokens -> output_tokens
   - Handle error responses (4xx, 5xx status codes) and preserve error format
   - Add convert_openai_to_claude_format() method following TypeScript anthropic.transformer.ts pattern

7. Error handling:
   - Network errors (connection, timeout)
   - HTTP errors (status codes)
   - JSON parsing errors
   - Provider-specific error formats
   - Log errors with provider context

8. **Transformer System Integration:**
   Apply provider-specific transformations using the modular transformer system:

   **Transformer Application:**
   - Use `crate::transformers::apply_transformer()` function for all transformations
   - Iterate through `provider.transformer.use_transformers` array from config
   - Handle both `TransformerUse::Simple(String)` and `TransformerUse::WithOptions(Vec<Value>)` formats
   - Apply transformations in order specified in config
   - Pass options object to transformers that support it (e.g., maxtoken)

   **Available Transformers:**
   - **"openrouter"** - OpenRouter/Groq compatibility (no system field, tool format conversion)
   - **"gemini"** - Google Gemini compatibility (with system field, tool format conversion)  
   - **["maxtoken", {"max_tokens": N}]** - Override max_tokens with specified value

   **Error Handling:**
   - Log warnings for unknown transformer names
   - Continue processing if individual transformer fails
   - Transformers are self-contained and don't affect each other

9. **Debug Logging:**
   - Log complete request being sent to provider (with pretty JSON formatting)
   - Log provider name and URL for each request
   - Use log::debug! for detailed request/response information
   - Use log::error! for HTTP errors with status codes

10. **Provider-specific handling:**
    - OpenAI-compatible endpoints (most providers)
    - Special cases for specific providers if needed
    - Header requirements per provider
    - Model name formatting

11. **Request format for OpenAI-compatible providers:**
    - POST to {api_base_url}/chat/completions (auto-append if missing)
    - Headers: Authorization: Bearer {api_key}, Content-Type: application/json
    - Body: {"model": model_name, "messages": [...], "tools": [...], "stream": false}

This creates the HTTP client with modular transformer support for forwarding routed requests to actual LLM providers.