use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SafetySettings {
    pub approval_mode: String,
    pub exec_timeout_seconds: u64,
    pub disable_dangerous_tools: bool,
    pub redact_secrets_in_logs: bool,
    pub auto_backup_before_changes: bool,
    pub family_mode: bool,
}

impl Default for SafetySettings {
    fn default() -> Self {
        SafetySettings {
            approval_mode: "smart".to_string(),
            exec_timeout_seconds: 60,
            disable_dangerous_tools: true,
            redact_secrets_in_logs: true,
            auto_backup_before_changes: true,
            family_mode: true,
        }
    }
}

pub async fn get_settings() -> anyhow::Result<SafetySettings> {
    let cfg = crate::config::get_config().await?;

    let approval_mode = cfg
        .get("approvals")
        .and_then(|a| a.get("exec"))
        .and_then(|e| e.get("mode"))
        .and_then(|m| m.as_str())
        .unwrap_or("smart")
        .to_string();

    let exec_timeout = cfg
        .get("tools")
        .and_then(|t| t.get("exec"))
        .and_then(|e| e.get("timeoutSec"))
        .and_then(|v| v.as_u64())
        .unwrap_or(60);

    let family = cfg
        .get("_shell")
        .and_then(|s| s.get("family_mode"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(SafetySettings {
        approval_mode,
        exec_timeout_seconds: exec_timeout,
        disable_dangerous_tools: family,
        redact_secrets_in_logs: true,
        auto_backup_before_changes: family,
        family_mode: family,
    })
}

/// Merge safety settings into existing config without destroying sibling keys.
pub async fn set_settings(settings: SafetySettings) -> anyhow::Result<()> {
    let mut cfg = crate::config::get_config().await?;

    // Merge approval mode (preserve other approvals sub-keys)
    if !cfg.get("approvals").is_some() {
        cfg["approvals"] = serde_json::json!({});
    }
    if !cfg["approvals"].get("exec").is_some() {
        cfg["approvals"]["exec"] = serde_json::json!({});
    }
    cfg["approvals"]["exec"]["mode"] = Value::String(settings.approval_mode.clone());

    // Merge exec timeout (preserve other tools sub-keys)
    if !cfg.get("tools").is_some() {
        cfg["tools"] = serde_json::json!({});
    }
    if !cfg["tools"].get("exec").is_some() {
        cfg["tools"]["exec"] = serde_json::json!({});
    }
    cfg["tools"]["exec"]["timeoutSec"] = Value::Number(settings.exec_timeout_seconds.into());

    // Shell metadata
    cfg["_shell"] = serde_json::json!({
        "family_mode": settings.family_mode,
        "redact_secrets": settings.redact_secrets_in_logs,
        "auto_backup": settings.auto_backup_before_changes
    });

    // Family mode: disable dangerous tools by default (only touch known risky entries)
    if settings.family_mode {
        if let Some(skills) = cfg.get_mut("skills")
            .and_then(|s| s.get_mut("entries"))
            .and_then(|e| e.as_object_mut())
        {
            let risky = ["discord", "slack", "xurl", "github", "gh-issues", "nano-pdf"];
            for r in &risky {
                if let Some(entry) = skills.get_mut(*r) {
                    entry["enabled"] = Value::Bool(false);
                }
            }
        }
    }

    crate::config::set_config(cfg).await?;

    // Learn preference
    let _ = crate::self_improve::record_preference(
        "family_mode",
        &settings.family_mode.to_string(),
        "safety"
    ).await;

    Ok(())
}
