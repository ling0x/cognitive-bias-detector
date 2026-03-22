use super::{AiDetectionResult};
use super::openai::parse_ai_response;
use crate::config::AiConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
    model: String,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
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
        .context("Anthropic API key not set. Add 'api_key' under [ai] in ~/.config/cbd/config.toml")?;

    let model = cfg.model.as_deref().unwrap_or("claude-3-haiku-20240307");
    let max_tokens = cfg.max_tokens.unwrap_or(1024);

    let client = Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": format!("{}\n\nIMPORTANT: Respond with ONLY valid JSON, no markdown fences.", system),
            "messages": [
                {"role": "user", "content": user}
            ]
        }))
        .send()
        .await
        .context("Failed to reach Anthropic API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Anthropic API error {}: {}", status, body);
    }

    let ant: AnthropicResponse = resp.json().await.context("Failed to parse Anthropic response")?;
    let content = ant
        .content
        .iter()
        .find(|b| b.kind == "text")
        .and_then(|b| b.text.as_deref())
        .unwrap_or("{}");

    // Strip any markdown code fences the model might add despite instructions
    let json_str = strip_code_fences(content);

    parse_ai_response(&json_str, "anthropic", &ant.model)
}

fn strip_code_fences(s: &str) -> String {
    let s = s.trim();
    let s = if s.starts_with("```") {
        let after_fence = s.trim_start_matches('`');
        // skip language label (e.g. "json\n")
        after_fence
            .find('\n')
            .map(|i| &after_fence[i + 1..])
            .unwrap_or(after_fence)
    } else {
        s
    };
    let s = if s.ends_with("```") {
        s.trim_end_matches('`').trim()
    } else {
        s
    };
    s.to_string()
}
