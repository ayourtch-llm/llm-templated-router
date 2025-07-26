use serde_json::{Value, json, Map};
use crate::router::{Message, ClaudeTool};
use std::collections::HashMap;

pub struct MessageTransformer;

impl MessageTransformer {
    pub fn transform_messages_to_openai(messages: &[Message]) -> Vec<Value> {
        let mut openai_messages = Vec::new();
        
        for message in messages {
            match message.role.as_str() {
                "user" => {
                    let (text_content, tool_results) = Self::process_user_content(&message.content);
                    
                    // Add user text content if any
                    if !text_content.is_empty() {
                        openai_messages.push(json!({
                            "role": "user",
                            "content": text_content
                        }));
                    }
                    
                    // Add tool messages for each tool result
                    for (tool_call_id, content, tool_name) in tool_results {
                        openai_messages.push(json!({
                            "role": "tool",
                            "content": content,
                            "tool_call_id": tool_call_id,
                            "name": tool_name
                        }));
                    }
                }
                "assistant" => {
                    let (text_content, tool_calls) = Self::process_assistant_content(&message.content);
                    
                    let mut msg = json!({
                        "role": "assistant",
                        "content": if text_content.is_empty() { Value::Null } else { Value::String(text_content) }
                    });
                    
                    if !tool_calls.is_empty() {
                        msg["tool_calls"] = Value::Array(tool_calls);
                    }
                    
                    openai_messages.push(msg);
                }
                _ => {
                    openai_messages.push(json!({
                        "role": message.role,
                        "content": Self::extract_text_content(&message.content)
                    }));
                }
            }
        }
        
        openai_messages
    }
    
    pub fn transform_tools_to_openai(tools: &[ClaudeTool]) -> Vec<Value> {
        tools.iter().map(|tool| {
            let description = if tool.description.is_empty() {
                ""
            } else {
                &tool.description
            };
            
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": description,
                    "parameters": tool.input_schema.clone()
                }
            })
        }).collect()
    }
    
    fn process_user_content(content: &Value) -> (String, Vec<(String, String, String)>) {
        let text = Self::extract_text_content(content);
        let tool_results = Self::extract_tool_results(content);
        (text, tool_results)
    }
    
    fn process_assistant_content(content: &Value) -> (String, Vec<Value>) {
        let text = Self::extract_text_content(content);
        let tool_calls = Self::extract_tool_calls(content);
        (text, tool_calls)
    }
    
    fn extract_text_content(content: &Value) -> String {
        match content {
            Value::String(s) => s.clone(),
            Value::Array(blocks) => {
                blocks.iter()
                    .filter_map(|block| {
                        if let Value::Object(map) = block {
                            if map.get("type").and_then(|t| t.as_str()) == Some("text") {
                                map.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }
            _ => String::new()
        }
    }
    
    fn extract_tool_calls(content: &Value) -> Vec<Value> {
        let mut tool_calls = Vec::new();
        
        if let Value::Array(blocks) = content {
            for block in blocks {
                if let Value::Object(map) = block {
                    if map.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                        let id = map.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_else(|| "call_000000000000000000000000");
                        
                        if let Some(name) = map.get("name").and_then(|n| n.as_str()) {
                            let empty_object = Value::Object(Map::new());
                            let input = map.get("input").unwrap_or(&empty_object);
                            
                            tool_calls.push(json!({
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": serde_json::to_string(input).unwrap_or_default()
                                }
                            }));
                        }
                    }
                }
            }
        }
        
        tool_calls
    }
    
    fn extract_tool_results(content: &Value) -> Vec<(String, String, String)> {
        let mut results = Vec::new();
        
        if let Value::Array(blocks) = content {
            for block in blocks {
                if let Value::Object(map) = block {
                    if map.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                        let tool_use_id = map.get("tool_use_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                            
                        let content = map.get("content")
                            .map(|c| Self::extract_text_content(c))
                            .unwrap_or_default();
                            
                        let name = map.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("tool")
                            .to_string();
                            
                        results.push((tool_use_id, content, name));
                    }
                }
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_simple_text_message() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: Value::String("Hello, world!".to_string())
        }];
        
        let result = MessageTransformer::transform_messages_to_openai(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "Hello, world!");
    }
    
    #[test]
    fn test_tool_use_message() {
        let messages = vec![Message {
            role: "assistant".to_string(),
            content: json!([
                {
                    "type": "text",
                    "text": "I'll help you search"
                },
                {
                    "type": "tool_use",
                    "id": "toolu_123",
                    "name": "search",
                    "input": {"query": "rust programming"}
                }
            ])
        }];
        
        let result = MessageTransformer::transform_messages_to_openai(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["content"], "I'll help you search");
        assert_eq!(result[0]["tool_calls"].as_array().unwrap().len(), 1);
        assert_eq!(result[0]["tool_calls"][0]["id"], "toolu_123");
        assert_eq!(result[0]["tool_calls"][0]["function"]["name"], "search");
    }
    
    #[test]
    fn test_tool_result_message() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: json!([
                {
                    "type": "text",
                    "text": "Here are the results"
                },
                {
                    "type": "tool_result",
                    "tool_use_id": "toolu_123",
                    "content": json!({"results": ["item1", "item2"]})
                }
            ])
        }];
        
        let result = MessageTransformer::transform_messages_to_openai(&messages);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "Here are the results");
        assert_eq!(result[1]["role"], "tool");
        assert_eq!(result[1]["tool_call_id"], "toolu_123");
    }
    
    #[test]
    fn test_tools_transformation() {
        let tools = vec![
            ClaudeTool {
                name: "search".to_string(),
                description: "Search the web".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    }
                })
            }
        ];
        
        let result = MessageTransformer::transform_tools_to_openai(&tools);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["type"], "function");
        assert_eq!(result[0]["function"]["name"], "search");
        assert_eq!(result[0]["function"]["parameters"]["type"], "object");
    }
    
    #[test]
    fn test_empty_description() {
        let tools = vec![
            ClaudeTool {
                name: "test".to_string(),
                description: "".to_string(),
                input_schema: json!({"type": "object"})
            }
        ];
        
        let result = MessageTransformer::transform_tools_to_openai(&tools);
        assert_eq!(result[0]["function"]["description"], "");
    }
}