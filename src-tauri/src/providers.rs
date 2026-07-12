use reqwest::Client;
use serde_json::{json, Value};

use crate::{
    models::{ProviderId, ProviderSettings, RewriteRequest},
    text::instruction_for,
};

pub async fn rewrite_with_provider(
    client: &Client,
    settings: &ProviderSettings,
    request: &RewriteRequest,
) -> Result<Option<String>, String> {
    if settings.provider == ProviderId::Offline {
        return Ok(None);
    }

    if settings.provider != ProviderId::Ollama
        && settings.api_key.as_deref().unwrap_or("").trim().is_empty()
    {
        return Ok(None);
    }

    let default_instruction = instruction_for(&request.mode, request.target_language.as_deref());
    let request_instruction = request.custom_prompt.as_deref().unwrap_or("").trim();
    let saved_instruction = settings.custom_prompt.trim();
    let instruction = if !request_instruction.is_empty() {
        request_instruction.to_string()
    } else if saved_instruction.is_empty() {
        default_instruction.to_string()
    } else {
        format!(
            "{default_instruction}\n\nUser rewrite instructions:\n{saved_instruction}"
        )
    };
    let prompt = format!(
        "{instruction}\n\nRules:\n- Return only the rewritten text.\n- Preserve the user's meaning.\n- Do not add explanations.\n\nText:\n{}",
        request.input
    );

    let output = match settings.provider {
        ProviderId::Openai => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.openai.com/v1/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Openrouter => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://openrouter.ai/api/v1/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Anthropic => call_anthropic(client, settings, &prompt).await?,
        ProviderId::Gemini => call_gemini(client, settings, &prompt).await?,
        ProviderId::Ollama => call_ollama(client, settings, &prompt).await?,
        ProviderId::Offline => return Ok(None),
    };

    Ok(Some(output))
}

async fn call_openai_compatible(
    client: &Client,
    settings: &ProviderSettings,
    endpoint: &str,
    prompt: &str,
) -> Result<String, String> {
    let response = client
        .post(endpoint)
        .bearer_auth(settings.api_key.as_deref().unwrap_or_default())
        .json(&json!({
            "model": settings.model.as_str(),
            "temperature": settings.temperature,
            "max_tokens": settings.max_tokens,
            "messages": [
                {
                    "role": "system",
                    "content": "You are CorteX, a precise desktop writing assistant."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    parse_json_response(
        response
            .json::<Value>()
            .await
            .map_err(|error| error.to_string())?,
        &["choices.0.message.content"],
    )
}

async fn call_anthropic(
    client: &Client,
    settings: &ProviderSettings,
    prompt: &str,
) -> Result<String, String> {
    let endpoint = settings
        .endpoint
        .as_deref()
        .unwrap_or("https://api.anthropic.com/v1/messages");
    let response = client
        .post(endpoint)
        .header("x-api-key", settings.api_key.as_deref().unwrap_or_default())
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": settings.model.as_str(),
            "max_tokens": settings.max_tokens,
            "temperature": settings.temperature,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    parse_json_response(
        response
            .json::<Value>()
            .await
            .map_err(|error| error.to_string())?,
        &["content.0.text"],
    )
}

async fn call_gemini(
    client: &Client,
    settings: &ProviderSettings,
    prompt: &str,
) -> Result<String, String> {
    let model = if settings.model.trim().is_empty() {
        "gemini-1.5-flash"
    } else {
        settings.model.as_str()
    };
    let endpoint = settings.endpoint.clone().unwrap_or_else(|| {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}",
            settings.api_key.as_deref().unwrap_or_default()
        )
    });

    let response = client
        .post(endpoint)
        .json(&json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": settings.temperature,
                "maxOutputTokens": settings.max_tokens
            }
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    parse_json_response(
        response
            .json::<Value>()
            .await
            .map_err(|error| error.to_string())?,
        &["candidates.0.content.parts.0.text"],
    )
}

async fn call_ollama(
    client: &Client,
    settings: &ProviderSettings,
    prompt: &str,
) -> Result<String, String> {
    let endpoint = settings
        .endpoint
        .as_deref()
        .unwrap_or("http://localhost:11434/api/generate");
    let response = client
        .post(endpoint)
        .json(&json!({
            "model": settings.model.as_str(),
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": settings.temperature
            }
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    parse_json_response(
        response
            .json::<Value>()
            .await
            .map_err(|error| error.to_string())?,
        &["response"],
    )
}

fn parse_json_response(value: Value, paths: &[&str]) -> Result<String, String> {
    for path in paths {
        if let Some(text) = read_path(&value, path) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    if let Some(error) = value.pointer("/error/message").and_then(Value::as_str) {
        return Err(error.to_string());
    }

    Err("Provider returned an empty response".to_string())
}

fn read_path<'a>(value: &'a Value, path: &str) -> Option<&'a str> {
    let mut current = value;
    for segment in path.split('.') {
        if let Ok(index) = segment.parse::<usize>() {
            current = current.get(index)?;
        } else {
            current = current.get(segment)?;
        }
    }
    current.as_str()
}
