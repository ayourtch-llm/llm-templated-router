pub mod openrouter_transformer;
pub mod gemini_transformer;
pub mod maxtoken_transformer;

use serde_json::Value;
use crate::server::ClaudeRequest;
use std::error::Error;

/// Common trait for provider-specific transformers that modify OpenAI format requests
pub trait ProviderTransformer {
    /// Apply provider-specific transformations to the request body
    fn transform(&self, body: &mut Value, claude_req: &ClaudeRequest) -> Result<(), Box<dyn Error>>;
    
    /// Get the transformer name for logging
    fn name(&self) -> &'static str;
}

/// Apply a single transformer to the request body
pub fn apply_transformer(
    transformer_name: &str,
    body: &mut Value,
    claude_req: &ClaudeRequest,
    options: Option<&Value>,
) -> Result<(), Box<dyn Error>> {
    match transformer_name {
        "openrouter" => {
            let transformer = openrouter_transformer::OpenRouterTransformer::new();
            transformer.transform(body, claude_req)
        }
        "gemini" => {
            let transformer = gemini_transformer::GeminiTransformer::new();
            transformer.transform(body, claude_req)
        }
        "maxtoken" => {
            let transformer = maxtoken_transformer::MaxTokenTransformer::new(options);
            transformer.transform(body, claude_req)
        }
        _ => {
            log::warn!("Unknown transformer: {}", transformer_name);
            Ok(())
        }
    }
}