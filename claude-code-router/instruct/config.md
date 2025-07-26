Create a configuration module that matches the TypeScript config structure exactly.

Based on config.example.json, create these data structures:

1. Config struct with these exact fields and serde renames:
   - providers: Vec<Provider> (with #[serde(rename = "Providers")])
   - router: RouterConfig (with #[serde(rename = "Router")])
   - apikey: Option<String> (with #[serde(rename = "APIKEY", default)])
   - host: Option<String> (with #[serde(rename = "HOST", default)])
   - log: Option<bool> (with #[serde(rename = "LOG", default)])

2. Provider struct with these exact fields:
   - name: String
   - api_base_url: String
   - api_key: String
   - models: Vec<String>
   - transformer: Option<TransformerConfig>

3. TransformerConfig struct with:
   - use_transformers: Vec<TransformerUse> (with #[serde(rename = "use")])

4. TransformerUse enum to handle both simple strings and arrays with options:
   - Simple(String) - for strings like "openrouter", "deepseek"  
   - WithOptions(Vec<serde_json::Value>) - for arrays like ["maxtoken", {"max_tokens": 16384}]
   - Use #[serde(untagged)] to automatically deserialize both formats

5. RouterConfig struct with all possible routing fields:
   - default: String
   - background: Option<String> (with #[serde(default)])
   - think: Option<String> (with #[serde(default)])
   - long_context: Option<String> (with #[serde(rename = "longContext", default)])
   - web_search: Option<String> (with #[serde(rename = "webSearch", default)])

6. Implement load_config() function that:
   - Reads from ~/.claude-code-router/config.json
   - Creates default config if file doesn't exist
   - Uses proper serde field renaming to match JSON structure exactly
   - Returns Result<Config, Box<dyn std::error::Error>>

7. Implement save_config() function that saves config to JSON file

8. Add all necessary serde derives and field renames to match JSON structure exactly

9. Use dirs crate to find home directory for config path

The config must be compatible with the existing TypeScript config format with proper field name mapping.