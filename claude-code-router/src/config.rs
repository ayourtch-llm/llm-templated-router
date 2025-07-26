use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "Providers")]
    pub providers: Vec<Provider>,
    #[serde(rename = "Router")]
    pub router: RouterConfig,
    #[serde(rename = "APIKEY", default)]
    pub apikey: Option<String>,
    #[serde(rename = "HOST", default)]
    pub host: Option<String>,
    #[serde(rename = "LOG", default)]
    pub log: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub transformer: Option<TransformerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    #[serde(rename = "use")]
    pub use_transformers: Vec<TransformerUse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransformerUse {
    Simple(String),
    WithConfig(String, serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub default: String,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub think: Option<String>,
    #[serde(rename = "longContext", default)]
    pub long_context: Option<String>,
    #[serde(rename = "webSearch", default)]
    pub web_search: Option<String>,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        let default_config = Config {
            providers: vec![],
            router: RouterConfig {
                default: "".to_string(),
                background: None,
                think: None,
                long_context: None,
                web_search: None,
            },
            apikey: None,
            host: None,
            log: Some(false),
        };
        
        save_config(&default_config)?;
        return Ok(default_config);
    }
    
    let config_content = fs::read_to_string(&config_path)?;
    let config: Config = serde_json::from_str(&config_content)?;
    
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, config_json)?;
    
    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = dirs::home_dir().ok_or("Could not find home directory")?;
    path.push(".claude-code-router");
    path.push("config.json");
    Ok(path)
}