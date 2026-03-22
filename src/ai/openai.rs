use super::{AiBiasEntry, AiDetectionResult};
use crate::config::AiConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

pub async fn analyse(
    _text: &str,
    system: &str,
    user: &str,
    cfg: &AiConfig,
) -> Result<AiDetectionResult> {
    let api_key = cfg
        .api_key
        .as_deref()
        .context("OpenAI API key not set. Add 'api_key' under [ai] in ~/.config/cbd/config.toml")?;

    let model = cfg.model.as_deref().unwrap_or("gpt-4o-mini");
    let base_url = cfg
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let max_tokens = cfg.max_tokens.unwrap_or(1024);

    let client = Client::new();
    let resp = client
        .post(format!("{}/chat/completions", base_url))
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "response_format": {"type": "json_object"}
        }))
        .send()
        .await
        .context("Failed to reach OpenAI API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API error {}: {}", status, body);
    }

    let oai: OpenAiResponse = resp.json().await.context("Failed to parse OpenAI response")?;
    let content = oai
        .choices
        .first()
        .map(|c| c.message.content.as_str())
        .unwrap_or("{}");

    parse_ai_response(content, "openai", &oai.model)
}

pub(super) fn parse_ai_response(
    json_str: &str,
    provider: &str,
    model: &str,
) -> Result<AiDetectionResult> {
    #[derive(Deserialize)]
    struct Raw {
        detected_biases: Option<Vec<RawBias>>,
        summary: Option<String>,
    }
    #[derive(Deserialize)]
    struct RawBias {
        name: Option<String>,
        confidence: Option<String>,
        reasoning: Option<String>,
        relevant_excerpt: Option<String>,
    }

    let raw: Raw = serde_json::from_str(json_str)
        .with_context(|| format!("Could not parse AI JSON response: {}", &json_str[..json_str.len().min(200)]))?;

    let detected = raw
        .detected_biases
        .unwrap_or_default()
        .into_iter()
        .map(|b| AiBiasEntry {
            name: b.name.unwrap_or_else(|| "Unknown".to_string()),
            confidence: b.confidence.unwrap_or_else(|| "Low".to_string()),
            reasoning: b.reasoning.unwrap_or_else(|| "No reasoning provided.".to_string()),
            relevant_excerpt: b.relevant_excerpt.unwrap_or_default(),
        })
        .collect();

    let analysis = format!(
        "AI ({} / {}) analysis completed.",
        provider, model
    );

    Ok(AiDetectionResult {
        provider: provider.to_string(),
        model: model.to_string(),
        analysis,
        detected_biases: detected,
        summary: raw.summary.unwrap_or_else(|| "No summary provided.".to_string()),
    })
}
