# Transformers Module Specification

Create a transformers module that provides provider-specific request transformations for LLM providers.

## Requirements

1. **Module Structure:**
   - Create `transformers/mod.rs` with public module declarations
   - Export individual transformer modules: `openrouter_transformer`, `gemini_transformer`, `maxtoken_transformer`
   - Provide common trait and utility functions

2. **ProviderTransformer Trait:**
   ```rust
   pub trait ProviderTransformer {
       fn transform(&self, body: &mut Value, claude_req: &ClaudeRequest) -> Result<(), Box<dyn Error>>;
       fn name(&self) -> &'static str;
   }
   ```

3. **Apply Transformer Function:**
   - `apply_transformer(transformer_name: &str, body: &mut Value, claude_req: &ClaudeRequest, options: Option<&Value>) -> Result<(), Box<dyn Error>>`
   - Route transformer name to appropriate transformer instance
   - Handle unknown transformers with warning logs
   - Support both simple transformers and transformers with options

4. **Transformer Types:**
   - **Simple transformers:** Applied with transformer name only (e.g., "openrouter", "gemini")
   - **Option transformers:** Applied with name and options object (e.g., ["maxtoken", {"max_tokens": 16384}])

5. **Error Handling:**
   - Graceful handling of unknown transformer names
   - Log warnings for unsupported transformers
   - Return Ok(()) for unknown transformers to avoid breaking the pipeline

6. **Dependencies:**
   - Import serde_json::Value for JSON manipulation
   - Import crate::server::ClaudeRequest for request context
   - Import std::error::Error for error handling

This module provides the common infrastructure for provider-specific request transformations.