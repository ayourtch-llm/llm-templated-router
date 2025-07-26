Create a configuration module that matches the TypeScript config structure exactly.

Based on config.example.json, create these data structures:

1. Config struct with these exact fields:
   - Providers: Vec<Provider>
   - Router: RouterConfig
   - APIKEY: String
   - HOST: String
   - LOG: Option<bool> (default false)

2. Provider struct with these fields:
   - name: String
   - api_base_url: String
   - api_key: String
   - models: Vec<String>
   - transformer: Option<TransformerConfig>

3. TransformerConfig struct with:
   - use_transformers: Vec<TransformerUse> (maps to "use" field in JSON)

4. TransformerUse enum:
   - Simple(String) - for strings like "openrouter", "deepseek"
   - WithConfig(String, serde_json::Value) - for arrays like ["maxtoken", {...}]

5. RouterConfig struct with:
   - default: String
   - background: String  
   - think: String
   - longContext: String (maps to "longContext" in JSON)

6. Implement load_config() function that:
   - Reads from ~/.claude-code-router/config.json
   - Creates default config if file doesn't exist
   - Uses proper serde field renaming for "use" -> "use_transformers"
   - Returns Result<Config, Box<dyn std::error::Error>>

7. Implement save_config() function that saves config to JSON file

8. Add all necessary serde derives and field renames to match JSON structure exactly

The config must be compatible with the existing TypeScript config format.