use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiKeyEntry {
    pub provider: String,
    pub name: String,
    pub has_key: bool,
    pub key_preview: Option<String>,
    pub valid_format: bool,
}

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("openai", "OpenAI", "sk-"),
    ("anthropic", "Anthropic", "sk-ant-"),
    ("openrouter", "OpenRouter", "sk-or-"),
    ("deepseek", "DeepSeek", "sk-"),
    ("gemini", "Google Gemini", ""),
    ("moonshot", "Moonshot", "sk-"),
    ("minimax", "MiniMax", ""),
    ("elevenlabs", "ElevenLabs", ""),
];

fn validate_key_format(provider: &str, key: &str) -> bool {
    match provider {
        "openai" => key.starts_with("sk-") && key.len() > 20,
        "anthropic" => key.starts_with("sk-ant-") && key.len() > 20,
        "openrouter" => key.starts_with("sk-or-") && key.len() > 20,
        "deepseek" => key.starts_with("sk-") && key.len() > 20,
        "moonshot" => key.starts_with("sk-") && key.len() > 20,
        "gemini" => key.len() > 10,
        "minimax" => key.len() > 10,
        "elevenlabs" => key.len() > 10,
        _ => key.len() > 5,
    }
}

pub async fn list_keys() -> anyhow::Result<Vec<ApiKeyEntry>> {
    let cfg = crate::config::get_config().await?;
    let mut entries = Vec::new();

    for (id, name, _prefix) in PROVIDERS {
        let has_key = cfg
            .get("models")
            .and_then(|m| m.get("providers"))
            .and_then(|p| p.get(id))
            .and_then(|prov| prov.get("apiKey"))
            .is_some();

        let (key_preview, valid) = if has_key {
            let raw = cfg
                .get("models")
                .and_then(|m| m.get("providers"))
                .and_then(|p| p.get(id))
                .and_then(|prov| prov.get("apiKey"))
                .and_then(|k| k.as_str())
                .unwrap_or("");
            let preview = if raw.len() > 8 {
                format!("{}...{}", &raw[..4], &raw[raw.len() - 4..])
            } else {
                "****".to_string()
            };
            (Some(preview), validate_key_format(id, raw))
        } else {
            (None, false)
        };

        entries.push(ApiKeyEntry {
            provider: id.to_string(),
            name: name.to_string(),
            has_key,
            key_preview,
            valid_format: valid,
        });
    }

    Ok(entries)
}

pub async fn set_key(provider: &str, key: &str) -> anyhow::Result<bool> {
    if !validate_key_format(provider, key) {
        return Ok(false);
    }

    let mut cfg = crate::config::get_config().await?;

    if !cfg.get("models").is_some() {
        cfg["models"] = serde_json::json!({ "providers": {} });
    }
    if !cfg["models"].get("providers").is_some() {
        cfg["models"]["providers"] = serde_json::json!({});
    }
    if !cfg["models"]["providers"].get(provider).is_some() {
        cfg["models"]["providers"][provider] = serde_json::json!({});
    }

    cfg["models"]["providers"][provider]["apiKey"] = Value::String(key.to_string());
    crate::config::set_config(cfg).await?;

    // Learn preference
    let _ = crate::self_improve::record_preference("preferred_provider", provider, "behavior").await;

    Ok(true)
}

pub async fn delete_key(provider: &str) -> anyhow::Result<()> {
    let mut cfg = crate::config::get_config().await?;
    if let Some(providers) = cfg["models"]["providers"].as_object_mut() {
        if let Some(prov) = providers.get_mut(provider) {
            if let Some(obj) = prov.as_object_mut() {
                obj.remove("apiKey");
            }
        }
    }
    crate::config::set_config(cfg).await?;
    Ok(())
}
