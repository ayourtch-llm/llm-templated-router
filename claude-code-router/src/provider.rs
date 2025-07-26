use serde_json::{json, Value};
use std::time::Duration;

use crate::config::Config;
use crate::router::RouterRequest;

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
        let url = format!("{}{}", provider.api_base_url.trim_end_matches('/'), "/chat/completions");
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
}