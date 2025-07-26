Create a routing module for claude-code-router that implements request routing logic.

Requirements:

1. Import existing Config from crate::config::Config (do not create new Config struct)

2. Create a Router struct with these methods:
   - new(config: crate::config::Config) -> Router
   - route_request(&self, request: &RouterRequest) -> Result<String, Box<dyn std::error::Error>>

3. Create a RouterRequest struct to represent incoming LLM requests:
   - model: Option<String>
   - messages: Vec<Message> 
   - system: Option<serde_json::Value>
   - tools: Option<Vec<Tool>>
   - thinking: Option<bool>

4. Create supporting structs:
   - Message with role: String, content: serde_json::Value
   - Tool with name: String, description: Option<String>, input_schema: Option<serde_json::Value>

5. Implement routing logic that mirrors the TypeScript version:
   - If model contains "," return it directly (provider,model format)
   - Calculate approximate token count from messages/system/tools
   - Route to config.router.long_context if token count > 60000 and it exists
   - Route to config.router.background for claude-3-5-haiku models if it exists
   - Route to config.router.think if thinking=true and it exists
   - Route to config.router.web_search if tools contain web_search type and it exists
   - Otherwise use config.router.default

6. Token counting logic (approximate):
   - Count characters in message content strings
   - Count characters in system prompts
   - Count characters in tool names/descriptions/schemas
   - Use rough 4-chars-per-token estimate (simpler than tiktoken)

7. Route parsing:
   - Parse route format "provider,model" -> (provider_name, model_name)
   - Return the selected route string

8. Error handling:
   - Graceful fallback to config.router.default on any errors
   - Use log::debug! for routing decisions

9. Use existing config structure fields:
   - config.router.default, config.router.background, etc.
   - config.providers for validation

Add proper imports for log, serde, and std collections as needed.