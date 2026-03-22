use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub ai: Option<AiConfig>,
    pub ui: Option<UiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    /// "openai" | "anthropic" | "gemini" | "ollama"
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    /// Base URL override (useful for Ollama)
    pub base_url: Option<String>,
    /// Max tokens to generate
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub show_examples: Option<bool>,
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cbd")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let cfg: Config = toml::from_str(&content)?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Example skeleton config for user reference
    pub fn example() -> &'static str {
        r#"# Cognitive Bias Detector — Configuration
# Location: ~/.config/cbd/config.toml

[ai]
# Provider: "openai" | "anthropic" | "gemini" | "ollama"
provider = "openai"
api_key = "sk-..."
model = "gpt-4o-mini"           # optional, provider default used otherwise
# base_url = "http://localhost:11434/v1"   # Ollama override
max_tokens = 1024

[ui]
# theme = "dark"   # future option
show_examples = true
"#
    }
}
