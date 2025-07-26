use serde_json::{json, Value};
use std::time::Duration;

use crate::config::Config;
use crate::router::RouterRequest;

#[derive(Clone)]
pub struct ProviderClient {
    client: reqwest::Client,
}

impl ProviderClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("router/0.1")
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    pub async fn send_request(
        &self,
        provider_route: &str,
        request: &RouterRequest,
        config: &Config,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // 1. Parse route
        let (provider_name, model_name) = provider_route
            .split_once(',')
            .ok_or("Invalid provider route format: expected \"provider,model\"")?;

        // 2. Find provider config
        let provider = config
            .providers
            .iter()
            .find(|p| p.name == provider_name)
            .ok_or_else(|| format!("Provider '{}' not found in config", provider_name))?;

        // 3. Build request
        let url = if provider.api_base_url.contains("/chat/completions") {
            provider.api_base_url.clone()
        } else {
            format!("{}{}", provider.api_base_url.trim_end_matches('/'), "/chat/completions")
        };
        let mut body = json!({
            "model": model_name,
            "messages": request.messages,
        });
        
        // Add optional fields
        if let Some(system) = &request.system {
            body["system"] = system.clone();
        }
        if let Some(tools) = &request.tools {
            body["tools"] = json!(tools);
        }

        let req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .bearer_auth(&provider.api_key);

        // 4. Send request
        let resp = req.json(&body).send().await?;
        let status = resp.status();

        // 5. Handle response
        if !status.is_success() {
            let error_text = resp.text().await?;
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        let json: Value = resp.json().await?;
        Ok(json)
    }
    
    // New method to handle full Claude request with all fields
    pub async fn send_claude_request(
        &self,
        provider_route: &str,
        claude_req: &crate::server::ClaudeRequest,
        config: &Config,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // 1. Parse route
        let (provider_name, model_name) = provider_route
            .split_once(',')
            .ok_or("Invalid provider route format: expected \"provider,model\"")?;

        // 2. Find provider config
        let provider = config
            .providers
            .iter()
            .find(|p| p.name == provider_name)
            .ok_or_else(|| format!("Provider '{}' not found in config", provider_name))?;

        // 3. Build request URL
        let url = if provider.api_base_url.contains("/chat/completions") {
            provider.api_base_url.clone()
        } else {
            format!("{}{}", provider.api_base_url.trim_end_matches('/'), "/chat/completions")
        };
        
        // 4. Build complete request body with all Claude Code fields
        let mut body = json!({
            "model": model_name,
            "messages": claude_req.messages,
        });
        
        // Add optional fields from Claude request
        if let Some(max_tokens) = claude_req.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(temperature) = claude_req.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(stream) = claude_req.stream {
            body["stream"] = json!(stream);
        }
        
        // Apply transformers to modify the request
        self.apply_transformers(&mut body, claude_req, provider)?;

        let req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .bearer_auth(&provider.api_key);

        // 5. Send request
        let resp = req.json(&body).send().await?;
        let status = resp.status();

        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            log::error!("HTTP {}: {}", status, error_text);
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        let json: Value = resp.json().await?;
        
        // Convert OpenAI response format to Claude format for compatibility
        let claude_response = self.convert_openai_to_claude_format(json)?;
        Ok(claude_response)
    }
    
    fn apply_transformers(
        &self,
        body: &mut Value,
        claude_req: &crate::server::ClaudeRequest,
        provider: &crate::config::Provider,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(transformer_config) = &provider.transformer {
            for transformer_use in &transformer_config.use_transformers {
                self.apply_transformer_use(body, claude_req, transformer_use)?;
            }
        }
        Ok(())
    }
    
    fn apply_transformer_use(
        &self,
        body: &mut Value,
        claude_req: &crate::server::ClaudeRequest,
        transformer_use: &crate::config::TransformerUse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match transformer_use {
            crate::config::TransformerUse::Simple(name) => {
                match name.as_str() {
                    "openrouter" => {
                        // OpenRouter transformer: Convert Claude format to OpenAI-compatible format
                        // Don't add system field - Groq doesn't support it
                        if let Some(tools) = &claude_req.tools {
                            body["tools"] = json!(tools);
                        }
                        // Note: system field is intentionally omitted for Groq compatibility
                    }
                    "gemini" => {
                        // Gemini transformer: Convert to Gemini API format
                        if let Some(system) = &claude_req.system {
                            body["system"] = system.clone();
                        }
                        if let Some(tools) = &claude_req.tools {
                            body["tools"] = json!(tools);
                        }
                    }
                    _ => {
                        log::warn!("Unknown transformer: {}", name);
                    }
                }
            }
            crate::config::TransformerUse::WithOptions(options_array) => {
                if options_array.len() >= 2 {
                    if let (Some(name), Some(options)) = (
                        options_array[0].as_str(),
                        options_array[1].as_object()
                    ) {
                        match name {
                            "maxtoken" => {
                                // MaxToken transformer: Override max_tokens
                                if let Some(max_tokens) = options.get("max_tokens") {
                                    body["max_tokens"] = max_tokens.clone();
                                }
                            }
                            _ => {
                                log::warn!("Unknown transformer with options: {}", name);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}