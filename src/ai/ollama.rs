use super::AiDetectionResult;
use super::openai::parse_ai_response;
use crate::config::AiConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
    model: String,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

pub async fn analyse(
    _text: &str,
    system: &str,
    user: &str,
    cfg: &AiConfig,
) -> Result<AiDetectionResult> {
    let model = cfg.model.as_deref().unwrap_or("llama3.2");
    let base_url = cfg
        .base_url
        .as_deref()
        .unwrap_or("http://localhost:11434");

    let system_with_json_hint = format!(
        "{}\n\nIMPORTANT: Your entire response must be valid JSON only. No explanations, no markdown fences.",
        system
    );

    let client = Client::new();
    let resp = client
        .post(format!("{}/api/chat", base_url))
        .json(&json!({
            "model": model,
            "stream": false,
            "messages": [
                {"role": "system", "content": system_with_json_hint},
                {"role": "user", "content": user}
            ],
            "format": "json"
        }))
        .send()
        .await
        .context("Failed to reach Ollama API — is Ollama running?")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama API error {}: {}", status, body);
    }

    let oll: OllamaResponse = resp
        .json()
        .await
        .context("Failed to parse Ollama response")?;

    parse_ai_response(&oll.message.content, "ollama", &oll.model)
}
