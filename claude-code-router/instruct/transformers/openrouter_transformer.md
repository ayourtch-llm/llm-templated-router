# OpenRouter Transformer Specification

Create an OpenRouter-specific transformer for provider API compatibility.

## Requirements

1. **OpenRouterTransformer Struct:**
   - Simple struct with `new()` constructor
   - Implements `ProviderTransformer` trait
   - Name: "openrouter"

2. **Tool Transformation Logic:**
   - Check if tools are already in OpenAI format (have "type": "function" field)
   - **If already in OpenAI format:** pass through unchanged to avoid double-transformation
   - **If in Claude format:** convert from Claude to OpenAI format:
     ```
     Claude: {"name": "...", "description": "...", "input_schema": {...}}
     OpenAI: {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
     ```
   - Map `input_schema` field to `parameters` field in function object
   - Handle empty or missing fields gracefully

3. **Groq Compatibility:**
   - **Do NOT add system field** - Groq doesn't support system field in request body
   - This is the key difference from other transformers
   - System prompt handling is done at the message level

4. **Implementation Requirements:**
   - Use serde_json for JSON manipulation
   - Handle tools array iteration safely with proper error handling
   - Log transformation activities for debugging
   - Preserve original structure for non-tool fields

5. **Test Coverage:**
   - Test Claude format tool transformation
   - Test OpenAI format tool pass-through (no double transformation)
   - Test empty tools array handling
   - Test missing tools field handling

6. **Usage Context:**
   - Used for Groq and OpenRouter providers
   - Applied when config specifies "openrouter" transformer
   - Part of provider-specific request preparation pipeline

This transformer ensures tools are in the correct OpenAI format while maintaining Groq compatibility by omitting system fields.