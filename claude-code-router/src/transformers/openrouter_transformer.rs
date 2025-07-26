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
        if let Some(tools) = body.get_mut("tools") {
            if let Some(tools_array) = tools.as_array_mut() {
                for tool in tools_array {
                    if let Some(tool_obj) = tool.as_object() {
                        // Check if tool is already in OpenAI format
                        if tool_obj.get("type").and_then(|t| t.as_str()) != Some("function") {
                            // Convert from Claude format to OpenAI format
                            let name = tool_obj.get("name").cloned().unwrap_or_else(|| json!(""));
                            let description = tool_obj.get("description").cloned().unwrap_or_else(|| json!(""));
                            let parameters = tool_obj.get("input_schema").cloned().unwrap_or_else(|| json!({}));
                            
                            *tool = json!({
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "description": description,
                                    "parameters": parameters
                                }
                            });
                        }
                    }
                }
            }
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
    
    #[test]
    fn test_empty_tools_array() {
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
            "tools": []
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        let tools = body["tools"].as_array().unwrap();
        assert!(tools.is_empty());
    }
    
    #[test]
    fn test_missing_tools_field() {
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
            "messages": []
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        assert!(body.get("tools").is_none());
    }
}