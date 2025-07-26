use log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Router {
    config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouterRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub system: Option<Value>,
    pub tools: Option<Vec<ClaudeTool>>,
    pub thinking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
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
            if let Some(ref long_context) = self.config.router.long_context {
                if !long_context.is_empty() {
                    return long_context.clone();
                }
            }
        }

        // 3. Background routing for haiku
        if let Some(model) = &request.model {
            if model.contains("claude-3-5-haiku") {
                if let Some(ref background) = self.config.router.background {
                    if !background.is_empty() {
                        return background.clone();
                    }
                }
            }
        }

        // 4. Thinking mode
        if request.thinking.unwrap_or(false) {
            if let Some(ref think) = self.config.router.think {
                if !think.is_empty() {
                    return think.clone();
                }
            }
        }

        // 5. Web search tools
        if let Some(tools) = &request.tools {
            if tools.iter().any(|t| t.name.starts_with("web_search")) {
                if let Some(ref web_search) = self.config.router.web_search {
                    if !web_search.is_empty() {
                        return web_search.clone();
                    }
                }
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
                        } else if let Value::Object(obj) = item {
                            if let Some(content) = obj.get("content") {
                                if let Some(s) = content.as_str() {
                                    chars += s.len();
                                }
                            }
                        }
                    }
                }
                Value::Object(obj) => {
                    if let Some(content) = obj.get("content") {
                        if let Some(s) = content.as_str() {
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
                chars += tool.description.len();
                chars += tool.input_schema.to_string().len();
            }
        }

        chars / 4
    }
}