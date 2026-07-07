use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillReview {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub risk_level: String, // "low", "medium", "high"
    pub safety_notes: Vec<String>,
    pub enabled: bool,
}

fn skills_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    Ok(home.join(".openclaw").join("skills"))
}

pub async fn list_skills() -> anyhow::Result<Vec<SkillReview>> {
    let dir = skills_dir()?;
    let mut skills = Vec::new();

    if !dir.exists() {
        return Ok(skills);
    }

    let mut entries = fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                let review = parse_skill_manifest(&path).await;
                skills.push(review);
            }
        }
    }

    Ok(skills)
}

async fn parse_skill_manifest(path: &PathBuf) -> SkillReview {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let skill_md = path.join("SKILL.md");

    let mut description = "No description available.".to_string();
    let mut permissions = Vec::new();
    let mut risk_level = "low".to_string();
    let mut safety_notes = Vec::new();

    if let Ok(content) = fs::read_to_string(&skill_md).await {
        // Cap read size to 1MB to prevent OOM from malicious huge files
        const MAX_SIZE: usize = 1_048_576;
        let content = if content.len() > MAX_SIZE {
            content[..MAX_SIZE].to_string()
        } else {
            content
        };
        // Parse YAML frontmatter if present
        if content.starts_with("---") {
            if let Some(end) = content[3..].find("---") {
                let frontmatter = &content[3..end + 3];
                if let Some(desc_line) = frontmatter.lines().find(|l| l.starts_with("description:")) {
                    description = desc_line["description:".len()..].trim().trim_matches('"').to_string();
                }
            }
        }

        // Scan for permission indicators
        let lower = content.to_lowercase();
        if lower.contains("bash") || lower.contains("shell") || lower.contains("exec") || lower.contains("command") {
            permissions.push("Shell command execution".to_string());
            risk_level = "high".to_string();
            safety_notes.push("This skill can run arbitrary shell commands.".to_string());
        }
        if lower.contains("file") || lower.contains("write") || lower.contains("delete") {
            permissions.push("File system access".to_string());
            if risk_level == "low" { risk_level = "medium".to_string(); }
            safety_notes.push("This skill can read/write files on your computer.".to_string());
        }
        if lower.contains("network") || lower.contains("http") || lower.contains("fetch") || lower.contains("api") {
            permissions.push("Network access".to_string());
            if risk_level == "low" { risk_level = "medium".to_string(); }
            safety_notes.push("This skill can make requests to the internet.".to_string());
        }
        if lower.contains("browser") || lower.contains("snapshot") || lower.contains("click") {
            permissions.push("Browser automation".to_string());
            if risk_level == "low" { risk_level = "medium".to_string(); }
            safety_notes.push("This skill can control your web browser.".to_string());
        }
    }

    if permissions.is_empty() {
        permissions.push("Read-only / information lookup".to_string());
        safety_notes.push("This skill appears to only read information.".to_string());
    }

    // Check if enabled in config
    let enabled = match crate::config::get_config().await {
        Ok(cfg) => cfg
            .get("skills")
            .and_then(|s| s.get("entries"))
            .and_then(|e| e.get(&name))
            .and_then(|entry| entry.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        Err(_) => true,
    };

    SkillReview {
        name,
        description,
        permissions,
        risk_level,
        safety_notes,
        enabled,
    }
}

pub async fn set_skill_enabled(name: String, enabled: bool) -> anyhow::Result<()> {
    let mut cfg = crate::config::get_config().await?;
    if !cfg.get("skills").is_some() {
        cfg["skills"] = serde_json::json!({ "entries": {} });
    }
    if !cfg["skills"].get("entries").is_some() {
        cfg["skills"]["entries"] = serde_json::json!({});
    }
    if !cfg["skills"]["entries"].get(&name).is_some() {
        cfg["skills"]["entries"][&name] = serde_json::json!({});
    }
    cfg["skills"]["entries"][&name]["enabled"] = serde_json::json!(enabled);
    crate::config::set_config(cfg).await?;
    Ok(())
}
