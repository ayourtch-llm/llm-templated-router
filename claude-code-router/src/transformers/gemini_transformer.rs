use serde_json::{json, Value};
use crate::server::ClaudeRequest;
use crate::transformers::ProviderTransformer;
use std::error::Error;

/// Gemini transformer: Converts to Gemini API format
/// Similar to OpenRouter but includes system field support
pub struct GeminiTransformer;

impl GeminiTransformer {
    pub fn new() -> Self {
        Self
    }
}

impl ProviderTransformer for GeminiTransformer {
    fn transform(&self, body: &mut Value, claude_req: &ClaudeRequest) -> Result<(), Box<dyn Error>> {
        // Add system field if present (Gemini supports this unlike Groq)
        if let Some(system) = &claude_req.system {
            body["system"] = system.clone();
        }
        
        // Transform tools if present (same logic as OpenRouter)
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
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "gemini"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_gemini_adds_system_field() {
        let transformer = GeminiTransformer::new();
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: Some(json!("You are a helpful assistant")),
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
        
        assert_eq!(body["system"], "You are a helpful assistant");
    }
    
    #[test]
    fn test_gemini_tools_transformation() {
        let transformer = GeminiTransformer::new();
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
}