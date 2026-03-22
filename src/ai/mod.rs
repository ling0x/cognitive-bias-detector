mod openai;
mod anthropic;
mod gemini;
mod ollama;
mod prompt;

use crate::config::AiConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use prompt::build_system_prompt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDetectionResult {
    pub provider: String,
    pub model: String,
    pub analysis: String,
    pub detected_biases: Vec<AiBiasEntry>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBiasEntry {
    pub name: String,
    pub confidence: String,
    pub reasoning: String,
    pub relevant_excerpt: String,
}

/// Dispatch to the correct AI provider
pub async fn analyse_with_ai(
    text: &str,
    provider: &str,
    cfg: &AiConfig,
) -> Result<AiDetectionResult> {
    let system = prompt::build_system_prompt();
    let user_prompt = prompt::build_user_prompt(text);

    match provider.to_lowercase().as_str() {
        "openai" => openai::analyse(text, &system, &user_prompt, cfg).await,
        "anthropic" => anthropic::analyse(text, &system, &user_prompt, cfg).await,
        "gemini" => gemini::analyse(text, &system, &user_prompt, cfg).await,
        "ollama" => ollama::analyse(text, &system, &user_prompt, cfg).await,
        other => anyhow::bail!("Unknown AI provider: '{}'. Supported: openai, anthropic, gemini, ollama", other),
    }
}
