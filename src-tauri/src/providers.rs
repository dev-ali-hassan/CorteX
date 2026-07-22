use reqwest::{Client, Response};
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
        format!("{default_instruction}\n\nUser rewrite instructions:\n{saved_instruction}")
    };
    let prompt = build_prompt(&instruction, &request.input);

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
        ProviderId::Groq => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.groq.com/openai/v1/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Mistral => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.mistral.ai/v1/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Xai => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.x.ai/v1/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Deepseek => {
            call_openai_compatible(
                client,
                settings,
                settings
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.deepseek.com/chat/completions"),
                &prompt,
            )
            .await?
        }
        ProviderId::Cohere => call_cohere(client, settings, &prompt).await?,
        ProviderId::Anthropic => call_anthropic(client, settings, &prompt).await?,
        ProviderId::Gemini => call_gemini(client, settings, &prompt).await?,
        ProviderId::Ollama => call_ollama(client, settings, &prompt).await?,
        ProviderId::Offline => return Ok(None),
    };

    Ok(Some(output))
}

fn build_prompt(instruction: &str, input: &str) -> String {
    format!(
        "You are an expert English editor.\n\nMode instruction:\n{instruction}\n\nMandatory editing baseline for every mode:\n\
- Correct every spelling, grammar, punctuation, capitalization, sentence-structure, and word-choice error before applying the mode instruction.\n\
- Capitalize the first word of every sentence, proper nouns, and acronyms such as AI, API, CPU, GPU, and USA.\n\
- Correct subject-verb agreement.\n\
- Correct confused words from context, including than/then, their/there, and your/you're.\n\
- Preserve the original meaning and do not invent information.\n\
- Make the result natural, fluent, and publication-quality.\n\
- Return only the final rewritten text: no explanation, markdown, labels, or quotation marks.\n\n\
Before responding, silently verify spelling, grammar, capitalization, punctuation, agreement, and word choice. If any issue remains, revise it.\n\nText:\n{input}"
    )
}

pub async fn test_connection(
    client: &Client,
    settings: &ProviderSettings,
) -> Result<String, String> {
    if settings.provider == ProviderId::Offline {
        return Err("Select an AI provider first.".to_string());
    }

    if settings.provider != ProviderId::Ollama
        && settings.api_key.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err("Enter an API key before testing the connection.".to_string());
    }

    match settings.provider {
        ProviderId::Openai
        | ProviderId::Openrouter
        | ProviderId::Groq
        | ProviderId::Mistral
        | ProviderId::Xai
        | ProviderId::Deepseek => {
            let default_endpoint = match settings.provider {
                ProviderId::Openai => "https://api.openai.com/v1/models",
                ProviderId::Openrouter => "https://openrouter.ai/api/v1/models",
                ProviderId::Groq => "https://api.groq.com/openai/v1/models",
                ProviderId::Mistral => "https://api.mistral.ai/v1/models",
                ProviderId::Xai => "https://api.x.ai/v1/models",
                ProviderId::Deepseek => "https://api.deepseek.com/models",
                _ => unreachable!(),
            };
            let endpoint = settings
                .endpoint
                .as_deref()
                .map(models_endpoint_from_openai_compatible)
                .unwrap_or_else(|| default_endpoint.to_string());
            ensure_success(
                client
                    .get(endpoint)
                    .bearer_auth(settings.api_key.as_deref().unwrap_or_default())
                    .send()
                    .await
                    .map_err(connection_error)?,
            )
            .await?;
        }
        ProviderId::Cohere => {
            let endpoint = settings
                .endpoint
                .as_deref()
                .map(|endpoint| models_endpoint_from_cohere(endpoint, &settings.model))
                .unwrap_or_else(|| format!("https://api.cohere.com/v1/models/{}", settings.model));
            ensure_success(
                client
                    .get(endpoint)
                    .bearer_auth(settings.api_key.as_deref().unwrap_or_default())
                    .send()
                    .await
                    .map_err(connection_error)?,
            )
            .await?;
        }
        ProviderId::Anthropic => {
            let endpoint = settings
                .endpoint
                .as_deref()
                .map(models_endpoint_from_anthropic)
                .unwrap_or_else(|| "https://api.anthropic.com/v1/models".to_string());
            ensure_success(
                client
                    .get(endpoint)
                    .header("x-api-key", settings.api_key.as_deref().unwrap_or_default())
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .await
                    .map_err(connection_error)?,
            )
            .await?;
        }
        ProviderId::Gemini => {
            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                settings.api_key.as_deref().unwrap_or_default()
            );
            ensure_success(
                client
                    .get(endpoint)
                    .send()
                    .await
                    .map_err(connection_error)?,
            )
            .await?;
        }
        ProviderId::Ollama => {
            let endpoint = settings
                .endpoint
                .as_deref()
                .map(ollama_tags_endpoint)
                .unwrap_or_else(|| "http://127.0.0.1:11434/api/tags".to_string());
            let value = response_json(
                client
                    .get(endpoint)
                    .send()
                    .await
                    .map_err(|error| {
                        if error.is_connect() {
                            "Ollama is not responding. Start Ollama and CorteX will reconnect automatically."
                                .to_string()
                        } else {
                            connection_error(error)
                        }
                    })?,
            )
            .await?;
            let configured_model = settings.model.trim();
            if !configured_model.is_empty() {
                let installed =
                    value
                        .get("models")
                        .and_then(Value::as_array)
                        .is_some_and(|models| {
                            models.iter().any(|model| {
                                let name = model
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default();
                                let model_id = model
                                    .get("model")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default();
                                name == configured_model
                                    || model_id == configured_model
                                    || name.split(':').next() == configured_model.split(':').next()
                            })
                        });
                if !installed {
                    return Err(format!(
                        "Ollama is running, but model '{configured_model}' is not installed."
                    ));
                }
            }
        }
        ProviderId::Offline => unreachable!(),
    }

    Ok("Connected".to_string())
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
        response_json(response).await?,
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

    parse_json_response(response_json(response).await?, &["content.0.text"])
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
        response_json(response).await?,
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
        .unwrap_or("http://127.0.0.1:11434/api/generate");
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

    parse_json_response(response_json(response).await?, &["response"])
}

async fn call_cohere(
    client: &Client,
    settings: &ProviderSettings,
    prompt: &str,
) -> Result<String, String> {
    let endpoint = settings
        .endpoint
        .as_deref()
        .unwrap_or("https://api.cohere.com/v2/chat");
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

    parse_json_response(response_json(response).await?, &["message.content.0.text"])
}

async fn response_json(response: Response) -> Result<Value, String> {
    let status = response.status();
    let value = response
        .json::<Value>()
        .await
        .map_err(|error| format!("Provider returned an unreadable response: {error}"))?;
    if status.is_success() {
        return Ok(value);
    }

    let message = value
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| value.get("error").and_then(Value::as_str))
        .unwrap_or("The provider rejected the connection request.");
    Err(format!("{} ({status})", message.trim()))
}

async fn ensure_success(response: Response) -> Result<(), String> {
    response_json(response).await.map(|_| ())
}

fn connection_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "Connection timed out. Check the provider address and try again.".to_string()
    } else if error.is_connect() {
        "Could not reach the provider. Check that it is running and the address is correct."
            .to_string()
    } else {
        format!("Connection failed: {error}")
    }
}

fn models_endpoint_from_openai_compatible(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(prefix) = endpoint.strip_suffix("/chat/completions") {
        format!("{prefix}/models")
    } else {
        format!("{endpoint}/models")
    }
}

fn models_endpoint_from_anthropic(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(prefix) = endpoint.strip_suffix("/messages") {
        format!("{prefix}/models")
    } else {
        format!("{endpoint}/models")
    }
}

fn models_endpoint_from_cohere(endpoint: &str, model: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(prefix) = endpoint.strip_suffix("/v2/chat") {
        format!("{prefix}/v1/models/{model}")
    } else {
        format!("{endpoint}/v1/models/{model}")
    }
}

fn ollama_tags_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(prefix) = endpoint.strip_suffix("/api/generate") {
        format!("{prefix}/api/tags")
    } else if endpoint.ends_with("/api/tags") {
        endpoint.to_string()
    } else {
        format!("{endpoint}/api/tags")
    }
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

#[cfg(test)]
mod tests {
    use super::{
        build_prompt, models_endpoint_from_anthropic, models_endpoint_from_cohere,
        models_endpoint_from_openai_compatible, ollama_tags_endpoint,
    };

    #[test]
    fn every_provider_prompt_enforces_the_editorial_baseline() {
        let prompt = build_prompt("Rewrite professionally.", "people does use ai");
        for requirement in [
            "Correct every spelling",
            "Capitalize the first word",
            "Correct subject-verb agreement",
            "than/then",
            "Preserve the original meaning",
            "Return only the final rewritten text",
            "silently verify spelling",
        ] {
            assert!(prompt.contains(requirement), "missing prompt rule: {requirement}");
        }
    }

    #[test]
    fn derives_openai_compatible_models_endpoint() {
        assert_eq!(
            models_endpoint_from_openai_compatible("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/models"
        );
    }

    #[test]
    fn derives_anthropic_models_endpoint() {
        assert_eq!(
            models_endpoint_from_anthropic("https://api.anthropic.com/v1/messages"),
            "https://api.anthropic.com/v1/models"
        );
    }

    #[test]
    fn derives_cohere_models_endpoint() {
        assert_eq!(
            models_endpoint_from_cohere("https://api.cohere.com/v2/chat", "command-a-plus-05-2026"),
            "https://api.cohere.com/v1/models/command-a-plus-05-2026"
        );
    }

    #[test]
    fn derives_ollama_tags_endpoint() {
        assert_eq!(
            ollama_tags_endpoint("http://localhost:11434/api/generate"),
            "http://localhost:11434/api/tags"
        );
    }
}
