use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Router {
    config: Config,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub router: RouterConfig,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct RouterConfig {
    pub default: String,
    pub long_context: String,
    pub background: String,
    pub think: String,
    pub web_search: String,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub models: HashMap<String, ModelConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouterRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub system: Option<Value>,
    pub tools: Option<Vec<Tool>>,
    pub thinking: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

impl Router {
    pub fn new(config: Config) -> Self {
        Router { config }
    }

    pub fn route_request(&self, request: &RouterRequest) -> Result<String, Box<dyn std::error::Error>> {
        let route = self.determine_route(request);
        log::debug!("Routing decision: {}", route);
        Ok(route)
    }

    fn determine_route(&self, request: &RouterRequest) -> String {
        // 1. Direct model specification
        if let Some(model) = &request.model {
            if model.contains(',') {
                return model.clone();
            }
        }

        // 2. Token count check
        let token_count = self.estimate_tokens(request);
        if token_count > 60_000 {
            return self.config.router.long_context.clone();
        }

        // 3. Background routing for haiku
        if let Some(model) = &request.model {
            if model.contains("claude-3-5-haiku") {
                return self.config.router.background.clone();
            }
        }

        // 4. Thinking mode
        if request.thinking.unwrap_or(false) {
            return self.config.router.think.clone();
        }

        // 5. Web search tools
        if let Some(tools) = &request.tools {
            if tools.iter().any(|t| t.name == "web_search") {
                return self.config.router.web_search.clone();
            }
        }

        // 6. Default route
        self.config.router.default.clone()
    }

    fn estimate_tokens(&self, request: &RouterRequest) -> usize {
        let mut chars = 0;

        // Messages
        for msg in &request.messages {
            match &msg.content {
                Value::String(s) => chars += s.len(),
                Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            chars += s.len();
                        }
                    }
                }
                _ => {}
            }
        }

        // System prompt
        if let Some(system) = &request.system {
            chars += system.to_string().len();
        }

        // Tools
        if let Some(tools) = &request.tools {
            for tool in tools {
                chars += tool.name.len();
                if let Some(desc) = &tool.description {
                    chars += desc.len();
                }
                if let Some(schema) = &tool.input_schema {
                    chars += schema.to_string().len();
                }
            }
        }

        chars / 4
    }
}