use super::AiDetectionResult;
use super::openai::parse_ai_response;
use crate::config::AiConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "modelVersion")]
    model_version: Option<String>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: Option<String>,
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
        .context("Gemini API key not set. Add 'api_key' under [ai] in ~/.config/cbd/config.toml")?;

    let model = cfg.model.as_deref().unwrap_or("gemini-1.5-flash");
    let max_tokens = cfg.max_tokens.unwrap_or(1024) as i64;

    let combined_prompt = format!(
        "{}\n\nIMPORTANT: Respond with ONLY valid JSON, no markdown fences.\n\n{}",
        system, user
    );

    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let resp = client
        .post(&url)
        .json(&json!({
            "contents": [{"parts": [{"text": combined_prompt}]}],
            "generationConfig": {
                "maxOutputTokens": max_tokens,
                "responseMimeType": "application/json"
            }
        }))
        .send()
        .await
        .context("Failed to reach Gemini API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Gemini API error {}: {}", status, body);
    }

    let gem: GeminiResponse = resp.json().await.context("Failed to parse Gemini response")?;
    let content = gem
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .and_then(|p| p.text.as_deref())
        .unwrap_or("{}");

    let model_name = gem.model_version.as_deref().unwrap_or(model);
    parse_ai_response(content, "gemini", model_name)
}
