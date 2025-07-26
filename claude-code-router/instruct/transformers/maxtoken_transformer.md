# MaxToken Transformer Specification

Create a MaxToken transformer that overrides max_tokens with configured values.

## Requirements

1. **MaxTokenTransformer Struct:**
   - Struct with `max_tokens: Option<u64>` field
   - Constructor `new(options: Option<&Value>) -> Self`
   - Implements `ProviderTransformer` trait
   - Name: "maxtoken"

2. **Options Parsing:**
   - Extract max_tokens value from options JSON object
   - Expected format: `{"max_tokens": 16384}`
   - Handle missing or invalid options gracefully
   - Store parsed value in struct field during construction

3. **Max Tokens Override Logic:**
   - If `max_tokens` value was parsed from options:
     - Override `body["max_tokens"]` with configured value
     - Log the override operation for debugging
   - If no valid `max_tokens` in options:
     - Log warning about missing value
     - Leave original max_tokens unchanged

4. **Configuration Usage:**
   - Used with TransformerUse::WithOptions format: `["maxtoken", {"max_tokens": 16384}]`
   - The options object (second array element) is passed to constructor
   - Applied to enforce provider-specific token limits

5. **Implementation Requirements:**
   - Parse options during construction, not during transformation
   - Use serde_json::Value for options handling
   - Handle u64 conversion safely with proper error handling
   - Log transformation activities for debugging
   - Graceful handling of missing or malformed options

6. **Test Coverage:**
   - Test valid max_tokens override from options
   - Test missing options (None) handling
   - Test invalid options format handling
   - Test options with wrong field names
   - Test numeric type conversion edge cases

7. **Usage Context:**
   - Used to enforce provider-specific token limits
   - Applied when config specifies ["maxtoken", {"max_tokens": N}] transformer
   - Commonly used to set higher limits for capable models
   - Part of provider-specific request preparation pipeline

8. **Error Handling:**
   - Never fail transformation due to invalid options
   - Log warnings for configuration issues
   - Preserve original max_tokens if override fails

This transformer provides flexible max_tokens override capability for provider-specific token limit requirements.