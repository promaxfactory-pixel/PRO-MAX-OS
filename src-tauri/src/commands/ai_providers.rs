use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use crate::commands::ai_assistant::load_setting;
use crate::db::DbState;
use crate::error::AppError;
use tauri::State;

pub const PROVIDER_CATALOG: &[ProviderInfo] = &[
    ProviderInfo {
        id: "openai",
        label: "OpenAI (GPT)",
        requires_key: true,
        default_model: "gpt-4o",
        models: &["gpt-4o", "gpt-4o-mini", "gpt-4.1", "gpt-4.1-mini"],
        base_url: "https://api.openai.com/v1",
        free_tier: false,
    },
    ProviderInfo {
        id: "anthropic",
        label: "Anthropic (Claude)",
        requires_key: true,
        default_model: "claude-3-5-sonnet-20241022",
        models: &["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"],
        base_url: "https://api.anthropic.com/v1",
        free_tier: false,
    },
    ProviderInfo {
        id: "gemini",
        label: "Google Gemini",
        requires_key: true,
        default_model: "gemini-2.0-flash",
        models: &["gemini-2.0-flash", "gemini-2.0-flash-lite", "gemini-1.5-flash", "gemini-1.5-pro"],
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        free_tier: true,
    },
    ProviderInfo {
        id: "groq",
        label: "Groq (Free LLMs)",
        requires_key: true,
        default_model: "llama-3.3-70b-versatile",
        models: &["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "mixtral-8x7b-32768", "llama-3.2-3b-preview"],
        base_url: "https://api.groq.com/openai/v1",
        free_tier: true,
    },
    ProviderInfo {
        id: "deepseek",
        label: "DeepSeek",
        requires_key: true,
        default_model: "deepseek-chat",
        models: &["deepseek-chat", "deepseek-reasoner"],
        base_url: "https://api.deepseek.com/v1",
        free_tier: false,
    },
    ProviderInfo {
        id: "mistral",
        label: "Mistral AI",
        requires_key: true,
        default_model: "mistral-small-latest",
        models: &["mistral-small-latest", "mistral-medium-latest", "mistral-large-latest"],
        base_url: "https://api.mistral.ai/v1",
        free_tier: true,
    },
    ProviderInfo {
        id: "ollama",
        label: "Ollama (Local / Offline)",
        requires_key: false,
        default_model: "llama3.2",
        models: &["llama3.2", "llama3.1", "qwen2.5", "gemma2", "mistral", "phi3"],
        base_url: "http://localhost:11434",
        free_tier: true,
    },
];

#[derive(Debug, Clone, Copy)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub requires_key: bool,
    pub default_model: &'static str,
    pub models: &'static [&'static str],
    pub base_url: &'static str,
    pub free_tier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub label: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
    pub enabled: bool,
    pub requires_key: bool,
    pub free_tier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub id: String,
    pub label: String,
    pub model: String,
    pub configured: bool,
    pub enabled: bool,
    pub requires_key: bool,
    pub free_tier: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub text: String,
    pub model: String,
    pub provider: String,
}

pub fn provider_catalog_map() -> Vec<serde_json::Value> {
    PROVIDER_CATALOG
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "label": p.label,
                "requires_key": p.requires_key,
                "default_model": p.default_model,
                "models": p.models,
                "base_url": p.base_url,
                "free_tier": p.free_tier,
            })
        })
        .collect()
}

pub fn load_provider_config(conn: &rusqlite::Connection, provider_id: &str) -> Result<ProviderConfig, AppError> {
    let info = PROVIDER_CATALOG
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| AppError::validation(format!("Unknown AI provider: {provider_id}")))?;

    let api_key = load_setting(conn, &format!("ai_api_key_{provider_id}"))?
        .or_else(|| load_setting(conn, "ai_api_key").ok().flatten())
        .map(|v| crate::crypto::decrypt_if_needed(&v).unwrap_or(v));

    let model = load_setting(conn, &format!("ai_model_{provider_id}"))?
        .or_else(|| load_setting(conn, "ai_model").ok().flatten())
        .unwrap_or_else(|| info.default_model.to_string());

    let base_url = load_setting(conn, &format!("ai_base_url_{provider_id}"))?
        .unwrap_or_else(|| info.base_url.to_string());

    let enabled = load_setting(conn, &format!("ai_enabled_{provider_id}"))?
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    Ok(ProviderConfig {
        id: info.id.to_string(),
        label: info.label.to_string(),
        api_key,
        model,
        base_url,
        enabled,
        requires_key: info.requires_key,
        free_tier: info.free_tier,
    })
}

pub fn is_provider_ready(cfg: &ProviderConfig) -> bool {
    if !cfg.enabled {
        return false;
    }
    if cfg.requires_key {
        cfg.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false)
    } else {
        true
    }
}

fn http_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::business(format!("Failed to create HTTP client: {e}")))
}

fn extract_openai_text(status: reqwest::StatusCode, json: &Value, provider: &str) -> Result<String, AppError> {
    if !status.is_success() {
        let err_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        return Err(AppError::business(format!("{provider} API error ({}): {}", status.as_u16(), err_msg)));
    }
    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::business(format!("No response content from {provider}")))
}

async fn call_openai_compatible(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
) -> Result<String, AppError> {
    let client = http_client()?;
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));

    let mut body = json!({
        "model": cfg.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    if json_mode {
        body["response_format"] = json!({"type": "json_object"});
    }

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json");
    if let Some(key) = &cfg.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::business(format!("{} API request failed: {e}", cfg.label)))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| AppError::business(format!("Failed to parse {} response: {e}", cfg.label)))?;

    extract_openai_text(status, &json, &cfg.label)
}

async fn call_anthropic(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
) -> Result<String, AppError> {
    let client = http_client()?;
    let url = format!("{}/messages", cfg.base_url.trim_end_matches('/'));

    let mut body = json!({
        "model": cfg.model,
        "system": system,
        "messages": [{"role": "user", "content": user}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    if json_mode {
        body["system"] = json!(format!("{system}\n\nCRITICAL: Respond with valid JSON only. No markdown, no prose outside the JSON."));
    }

    let resp = client
        .post(&url)
        .header("x-api-key", cfg.api_key.as_deref().unwrap_or(""))
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::business(format!("{} API request failed: {e}", cfg.label)))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| AppError::business(format!("Failed to parse {} response: {e}", cfg.label)))?;

    if !status.is_success() {
        let err_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        return Err(AppError::business(format!("{} API error ({}): {}", cfg.label, status.as_u16(), err_msg)));
    }

    let text = json["content"]
        .as_array()
        .and_then(|arr| arr.iter().find(|c| c["type"] == "text"))
        .and_then(|c| c["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::business(format!("No response content from {}", cfg.label)))?;

    Ok(text)
}

async fn call_gemini(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
) -> Result<String, AppError> {
    let client = http_client()?;
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        cfg.base_url.trim_end_matches('/'),
        cfg.model,
        cfg.api_key.as_deref().unwrap_or("")
    );

    let contents = json!([
        {"parts": [{"text": system}]},
        {"parts": [{"text": user}]}
    ]);

    let mut body = json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": max_tokens,
            "temperature": temperature,
        }
    });

    if json_mode {
        body["generationConfig"]["responseMimeType"] = json!("application/json");
        body["systemInstruction"] = json!({"parts": [{"text": system}]});
        body["contents"] = json!([{"parts": [{"text": user}]}]);
    }

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::business(format!("{} API request failed: {e}", cfg.label)))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| AppError::business(format!("Failed to parse {} response: {e}", cfg.label)))?;

    if !status.is_success() {
        let err_msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
        return Err(AppError::business(format!("{} API error ({}): {}", cfg.label, status.as_u16(), err_msg)));
    }

    json["candidates"][0]["content"]["parts"]
        .as_array()
        .and_then(|parts| parts.iter().find_map(|p| p["text"].as_str()))
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::business(format!("No response content from {}", cfg.label)))
}

async fn call_ollama(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    json_mode: bool,
) -> Result<String, AppError> {
    let client = http_client()?;
    let url = format!("{}/api/chat", cfg.base_url.trim_end_matches('/'));

    let mut body = json!({
        "model": cfg.model,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "options": {
            "temperature": 0.2
        }
    });

    if json_mode {
        body["format"] = json!("json");
    }

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::business(format!("{} API request failed: {e} (is Ollama running?)", cfg.label)))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| AppError::business(format!("Failed to parse {} response: {e}", cfg.label)))?;

    if !status.is_success() {
        let err_msg = json["error"].as_str().unwrap_or("Unknown error");
        return Err(AppError::business(format!("{} API error ({}): {}", cfg.label, status.as_u16(), err_msg)));
    }

    json["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::business(format!("No response content from {}", cfg.label)))
}

pub async fn call_provider(
    cfg: &ProviderConfig,
    system: &str,
    user: &str,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
) -> Result<ProviderResponse, AppError> {
    if !is_provider_ready(cfg) {
        let msg = if cfg.requires_key {
            format!("{} is not configured. Add an API key in Settings > AI.", cfg.label)
        } else {
            format!("{} is disabled.", cfg.label)
        };
        return Err(AppError::validation(msg));
    }

    let text = match cfg.id.as_str() {
        "anthropic" => call_anthropic(cfg, system, user, max_tokens, temperature, json_mode).await?,
        "gemini" => call_gemini(cfg, system, user, max_tokens, temperature, json_mode).await?,
        "ollama" => call_ollama(cfg, system, user, json_mode).await?,
        _ => call_openai_compatible(cfg, system, user, max_tokens, temperature, json_mode).await?,
    };

    Ok(ProviderResponse {
        text,
        model: cfg.model.clone(),
        provider: cfg.id.clone(),
    })
}

fn strip_markdown_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    let t = trimmed.trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
    if !t.is_empty() { t.to_string() } else { trimmed.to_string() }
}

pub fn parse_json_response(text: &str) -> Result<Value, AppError> {
    let cleaned = strip_markdown_code_fences(text);
    serde_json::from_str(&cleaned)
        .map_err(|_| AppError::business("AI returned invalid JSON. Please retry.".to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverResult {
    pub text: String,
    pub model: String,
    pub provider: String,
    pub provider_label: String,
    pub used_fallback: bool,
    pub attempts: Vec<String>,
}

pub async fn chat_with_failover(
    state: State<'_, DbState>,
    system: &str,
    user: &str,
    max_tokens: i64,
    temperature: f64,
    json_mode: bool,
) -> Result<FailoverResult, AppError> {
    let order = ["ollama", "groq", "gemini", "deepseek", "mistral", "openai", "anthropic"];
    let mut attempts: Vec<String> = Vec::new();
    let mut last_error: Option<String> = None;

    let configs = {
        let conn = state.0.lock()?;
        let mut out = Vec::new();
        for id in order {
            let cfg = load_provider_config(&conn, id)?;
            if is_provider_ready(&cfg) {
                out.push(cfg);
            }
        }
        out
    };

    if configs.is_empty() {
        return Err(AppError::validation(
            "No AI provider configured. Add an API key or start Ollama (Settings > AI Integration).".to_string(),
        ));
    }

    for cfg in &configs {
        attempts.push(cfg.id.clone());
        match call_provider(cfg, system, user, max_tokens, temperature, json_mode).await {
            Ok(resp) => {
                let provider_label = PROVIDER_CATALOG
                    .iter()
                    .find(|p| p.id == cfg.id)
                    .map(|p| p.label.to_string())
                    .unwrap_or_else(|| cfg.label.clone());
                return Ok(FailoverResult {
                    text: resp.text,
                    model: resp.model,
                    provider: resp.provider,
                    provider_label,
                    used_fallback: attempts.len() > 1,
                    attempts,
                });
            }
            Err(e) => {
                last_error = Some(format!("{}: {e}", cfg.label));
            }
        }
    }

    Err(AppError::business(format!(
        "All AI providers failed.\n{}",
        last_error.unwrap_or_else(|| "No providers available.".to_string())
    )))
}

pub async fn chat_with_failover_json(
    state: State<'_, DbState>,
    system: &str,
    user: &str,
    max_tokens: i64,
) -> Result<Value, AppError> {
    let result = chat_with_failover(state, system, user, max_tokens, 0.1, true).await?;
    parse_json_response(&result.text)
}

// ─── Tauri commands ────────────────────────────────────────────────

#[tauri::command]
pub fn ai_provider_catalog() -> Vec<serde_json::Value> {
    provider_catalog_map()
}

#[tauri::command]
pub fn ai_provider_statuses(state: State<'_, DbState>) -> Result<Vec<ProviderStatus>, AppError> {
    let conn = state.0.lock()?;
    let mut out = Vec::new();
    for info in PROVIDER_CATALOG {
        let cfg = load_provider_config(&conn, info.id)?;
        let ready = is_provider_ready(&cfg);
        out.push(ProviderStatus {
            id: cfg.id,
            label: cfg.label,
            model: cfg.model,
            configured: ready,
            enabled: cfg.enabled,
            requires_key: cfg.requires_key,
            free_tier: cfg.free_tier,
            message: if ready {
                "Ready".to_string()
            } else if cfg.requires_key {
                "API key required".to_string()
            } else {
                "Disabled".to_string()
            },
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn ai_get_provider_settings(state: State<'_, DbState>, provider: String) -> Result<serde_json::Value, AppError> {
    let conn = state.0.lock()?;
    let cfg = load_provider_config(&conn, &provider)?;
    Ok(json!({
        "provider": cfg.id,
        "label": cfg.label,
        "model": cfg.model,
        "base_url": cfg.base_url,
        "enabled": cfg.enabled,
        "has_key": cfg.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false),
        "requires_key": cfg.requires_key,
        "models": PROVIDER_CATALOG.iter().find(|p| p.id == provider).map(|p| p.models).unwrap_or(&[]),
    }))
}

#[tauri::command]
pub fn ai_save_provider_config(
    state: State<'_, DbState>,
    provider: String,
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    enabled: Option<bool>,
) -> Result<String, AppError> {
    let conn = state.0.lock()?;
    if PROVIDER_CATALOG.iter().find(|p| p.id == provider).is_none() {
        return Err(AppError::validation(format!("Unknown AI provider: {provider}")));
    }
    if let Some(key) = &api_key {
        if !key.is_empty() {
            let encrypted = crate::crypto::encrypt_if_needed(key)
                .map_err(|e| AppError::business(format!("Failed to encrypt API key: {e}")))?;
            crate::commands::ai_assistant::save_setting(&conn, &format!("ai_api_key_{provider}"), &encrypted)?;
        }
    }
    if let Some(m) = &model {
        if !m.is_empty() {
            crate::commands::ai_assistant::save_setting(&conn, &format!("ai_model_{provider}"), m)?;
        }
    }
    if let Some(u) = &base_url {
        if !u.is_empty() {
            crate::commands::ai_assistant::save_setting(&conn, &format!("ai_base_url_{provider}"), u)?;
        }
    }
    if let Some(e) = enabled {
        crate::commands::ai_assistant::save_setting(&conn, &format!("ai_enabled_{provider}"), if e { "1" } else { "0" })?;
    }
    Ok(format!("{provider} settings saved"))
}

#[tauri::command]
pub async fn ai_test_provider(state: State<'_, DbState>, provider: String) -> Result<ProviderStatus, AppError> {
    let cfg = {
        let conn = state.0.lock()?;
        load_provider_config(&conn, &provider)?
    };
    let label = cfg.label.clone();
    let ready = is_provider_ready(&cfg);
    let msg = if !ready {
        if cfg.requires_key {
            "API key required".to_string()
        } else {
            "Disabled".to_string()
        }
    } else {
        match call_provider(&cfg, "Reply with exactly: OK", "Connection test", 16, 0.0, false).await {
            Ok(_) => "Connection successful!".to_string(),
            Err(e) => format!("Connection failed: {e}"),
        }
    };
    Ok(ProviderStatus {
        id: cfg.id,
        label,
        model: cfg.model,
        configured: ready,
        enabled: cfg.enabled,
        requires_key: cfg.requires_key,
        free_tier: cfg.free_tier,
        message: msg,
    })
}

#[tauri::command]
pub async fn ai_failover_chat(
    state: State<'_, DbState>,
    system: String,
    user: String,
) -> Result<FailoverResult, AppError> {
    chat_with_failover(state, &system, &user, 2048, 0.7, false).await
}

#[tauri::command]
pub async fn ai_get_available_models(provider: String) -> Result<Vec<String>, AppError> {
    if let Some(info) = PROVIDER_CATALOG.iter().find(|p| p.id == provider) {
        return Ok(info.models.iter().map(|m| m.to_string()).collect());
    }
    Err(AppError::validation(format!("Unknown provider: {provider}")))
}
