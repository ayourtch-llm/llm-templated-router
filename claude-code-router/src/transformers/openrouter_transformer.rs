use serde_json::{json, Value};
use crate::server::ClaudeRequest;
use crate::transformers::ProviderTransformer;
use std::error::Error;

/// OpenRouter transformer: Ensures tools are in OpenAI format
/// Specifically designed for Groq compatibility (no system field support)
pub struct OpenRouterTransformer;

impl OpenRouterTransformer {
    pub fn new() -> Self {
        Self
    }
}

impl ProviderTransformer for OpenRouterTransformer {
    fn transform(&self, body: &mut Value, _claude_req: &ClaudeRequest) -> Result<(), Box<dyn Error>> {
        // Transform tools if present
        if let Some(tools) = body.get("tools") {
            let empty_vec = vec![];
            let tools_array = tools.as_array().unwrap_or(&empty_vec);
            let openai_tools: Vec<Value> = tools_array.iter().map(|tool| {
                let tool_obj = tool.as_object().unwrap();
                
                // Check if tool is already in OpenAI format
                if tool_obj.get("type").and_then(|t| t.as_str()) == Some("function") {
                    // Already in OpenAI format, pass through
                    tool.clone()
                } else {
                    // Convert from Claude format to OpenAI format
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool_obj.get("name").unwrap_or(&json!("")),
                            "description": tool_obj.get("description").unwrap_or(&json!("")),
                            "parameters": tool_obj.get("input_schema").unwrap_or(&json!({}))
                        }
                    })
                }
            }).collect();
            body["tools"] = json!(openai_tools);
        }
        
        // Note: system field is intentionally omitted for Groq compatibility
        // OpenRouter/Groq doesn't support the system field in the request body
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "openrouter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_openrouter_tools_transformation() {
        let transformer = OpenRouterTransformer::new();
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: None,
            tools: None,
            thinking: None,
            max_tokens: None,
            temperature: None,
            stream: None,
            metadata: None,
        };
        
        let mut body = json!({
            "model": "test",
            "messages": [],
            "tools": [
                {
                    "name": "search",
                    "description": "Search the web",
                    "input_schema": {"type": "object"}
                }
            ]
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        let tools = body["tools"].as_array().unwrap();
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "search");
        assert_eq!(tools[0]["function"]["description"], "Search the web");
    }
    
    #[test]
    fn test_openrouter_already_openai_format() {
        let transformer = OpenRouterTransformer::new();
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: None,
            tools: None,
            thinking: None,
            max_tokens: None,
            temperature: None,
            stream: None,
            metadata: None,
        };
        
        let mut body = json!({
            "model": "test",
            "messages": [],
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "search",
                        "description": "Search the web",
                        "parameters": {"type": "object"}
                    }
                }
            ]
        });
        
        let original_tools = body["tools"].clone();
        transformer.transform(&mut body, &claude_req).unwrap();
        
        // Should pass through unchanged
        assert_eq!(body["tools"], original_tools);
    }
}