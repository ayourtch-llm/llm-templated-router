# Message Transformer Module

Create a message transformation module that converts Claude API message format to OpenAI-compatible format for LLM providers.

## Requirements

1. **Import required modules:**
   - serde_json::{Value, json} for JSON manipulation
   - crate::router::{Message, ClaudeTool} for type definitions
   - std::collections::HashMap for efficient lookups

2. **Create MessageTransformer struct:**
   - Stateless transformer for converting message formats
   - Methods for handling different transformation scenarios

3. **Core transformation functions:**

   **transform_messages_to_openai(messages: &[Message]) -> Vec<Value>:**
   - Convert Claude message format to OpenAI chat completion format
   - Handle complex content blocks (text, tool_use, tool_result)
   - Transform multi-turn tool conversations correctly
   - Preserve message roles and content appropriately

   **Key transformations:**
   - **User messages with tool_result content blocks:**
     - Extract tool_result blocks and convert to separate tool messages (role: "tool")
     - Preserve text content in user message
     - Set tool_call_id from tool_result.tool_use_id
     - Set name from tool_result.tool_use_id or function name

   - **Assistant messages with tool_use content blocks:**
     - Extract tool_use blocks and convert to tool_calls array in OpenAI format
     - Preserve text content in assistant message content
     - Transform tool_use to: {"id": tool_use.id, "type": "function", "function": {"name": tool_use.name, "arguments": JSON.stringify(tool_use.input)}}

   - **Simple text messages:**
     - Pass through with role and content preserved
     - Handle both string content and array content formats

4. **Tool format transformation:**

   **transform_tools_to_openai(tools: &[ClaudeTool]) -> Vec<Value>:**
   - Convert Claude tool format to OpenAI function calling format
   - Claude format: {"name": "...", "description": "...", "input_schema": {...}}
   - OpenAI format: {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
   - Rename input_schema to parameters

5. **Content block handling:**

   **extract_text_content(content: &Value) -> String:**
   - Extract text from complex content arrays
   - Handle both string content and array of content blocks
   - Concatenate text blocks, ignore non-text content types

   **extract_tool_calls(content: &Value) -> Vec<Value>:**
   - Extract tool_use blocks from content arrays
   - Convert to OpenAI tool_calls format
   - Generate unique tool call IDs if missing

   **extract_tool_results(content: &Value) -> Vec<(String, String, String)>:**
   - Extract tool_result blocks from content arrays  
   - Return tuples of (tool_call_id, content, tool_name)
   - Use tool_use_id as tool_call_id for correlation

6. **Error handling:**
   - Graceful handling of malformed content blocks
   - Default to text content extraction when structure is unexpected
   - Log warnings for unsupported content types
   - Never panic on malformed input

7. **Implementation patterns:**
   - Follow the TypeScript anthropic.transformer.ts patterns
   - Handle edge cases like mixed content blocks
   - Preserve conversation flow and tool call/result correlation
   - Support streaming and non-streaming scenarios

8. **Test support:**
   - Include unit tests for common transformation scenarios
   - Test tool call/result correlation
   - Test mixed content message handling
   - Test malformed input resilience

This transformer bridges the gap between Claude's rich content model and OpenAI's simpler message format, enabling seamless provider integration while preserving tool conversation semantics.