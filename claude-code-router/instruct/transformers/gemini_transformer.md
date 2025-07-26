# Gemini Transformer Specification

Create a Gemini-specific transformer for Google Gemini API compatibility.

## Requirements

1. **GeminiTransformer Struct:**
   - Simple struct with `new()` constructor
   - Implements `ProviderTransformer` trait
   - Name: "gemini"

2. **System Field Support:**
   - **Add system field** from `claude_req.system` if present
   - Unlike OpenRouter, Gemini supports system fields in request body
   - Set `body["system"] = claude_req.system.clone()`

3. **Tool Transformation Logic:**
   - Identical to OpenRouter transformer for tool handling
   - Check if tools are already in OpenAI format (have "type": "function" field)
   - **If already in OpenAI format:** pass through unchanged
   - **If in Claude format:** convert from Claude to OpenAI format:
     ```
     Claude: {"name": "...", "description": "...", "input_schema": {...}}
     OpenAI: {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
     ```
   - Map `input_schema` field to `parameters` field in function object

4. **Key Differences from OpenRouter:**
   - **Includes system field support** (main difference)
   - Otherwise identical tool transformation logic
   - Used for Google Gemini providers

5. **Implementation Requirements:**
   - Use serde_json for JSON manipulation
   - Handle tools array iteration safely with proper error handling
   - Add system field before tool transformation
   - Preserve original structure for other fields

6. **Test Coverage:**
   - Test system field addition from claude_req.system
   - Test Claude format tool transformation  
   - Test OpenAI format tool pass-through
   - Test system field handling when None
   - Test combined system + tools transformation

7. **Usage Context:**
   - Used for Google Gemini providers
   - Applied when config specifies "gemini" transformer
   - Part of provider-specific request preparation pipeline

This transformer provides Gemini API compatibility by supporting both system fields and proper tool format transformation.