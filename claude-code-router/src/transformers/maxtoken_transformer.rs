use serde_json::Value;
use crate::server::ClaudeRequest;
use crate::transformers::ProviderTransformer;
use std::error::Error;

/// MaxToken transformer: Overrides max_tokens with configured value
/// Used with TransformerUse::WithOptions format: ["maxtoken", {"max_tokens": 16384}]
pub struct MaxTokenTransformer {
    max_tokens: Option<u64>,
}

impl MaxTokenTransformer {
    pub fn new(options: Option<&Value>) -> Self {
        let max_tokens = options
            .and_then(|opts| opts.as_object())
            .and_then(|obj| obj.get("max_tokens"))
            .and_then(|v| v.as_u64());
            
        Self { max_tokens }
    }
}

impl ProviderTransformer for MaxTokenTransformer {
    fn transform(&self, body: &mut Value, _claude_req: &ClaudeRequest) -> Result<(), Box<dyn Error>> {
        if let Some(max_tokens) = self.max_tokens {
            body["max_tokens"] = Value::Number(max_tokens.into());
            log::debug!("MaxToken transformer: Set max_tokens to {}", max_tokens);
        } else {
            log::warn!("MaxToken transformer: No max_tokens value provided in options");
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "maxtoken"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_maxtoken_sets_value() {
        let options = json!({"max_tokens": 16384});
        let transformer = MaxTokenTransformer::new(Some(&options));
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: None,
            tools: None,
            thinking: None,
            max_tokens: Some(512), // This should be overridden
            temperature: None,
            stream: None,
            metadata: None,
        };
        
        let mut body = json!({
            "model": "test",
            "messages": [],
            "max_tokens": 512
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        assert_eq!(body["max_tokens"], 16384);
    }
    
    #[test]
    fn test_maxtoken_no_options() {
        let transformer = MaxTokenTransformer::new(None);
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: None,
            tools: None,
            thinking: None,
            max_tokens: Some(512),
            temperature: None,
            stream: None,
            metadata: None,
        };
        
        let mut body = json!({
            "model": "test",
            "messages": [],
            "max_tokens": 512
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        // Should remain unchanged
        assert_eq!(body["max_tokens"], 512);
    }
    
    #[test]
    fn test_maxtoken_invalid_options() {
        let options = json!({"wrong_field": 16384});
        let transformer = MaxTokenTransformer::new(Some(&options));
        let claude_req = ClaudeRequest {
            model: "test".to_string(),
            messages: vec![],
            system: None,
            tools: None,
            thinking: None,
            max_tokens: Some(512),
            temperature: None,
            stream: None,
            metadata: None,
        };
        
        let mut body = json!({
            "model": "test",
            "messages": [],
            "max_tokens": 512
        });
        
        transformer.transform(&mut body, &claude_req).unwrap();
        
        // Should remain unchanged due to missing max_tokens field
        assert_eq!(body["max_tokens"], 512);
    }
}