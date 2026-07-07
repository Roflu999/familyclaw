use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

fn config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    Ok(home.join(".openclaw").join("openclaw.json"))
}

fn backup_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    Ok(home.join(".openclaw").join("openclaw.json.shell.bak"))
}

/// Deep-merge `src` into `dst`. Arrays are replaced; objects are merged recursively.
fn deep_merge(dst: &mut Value, src: Value) {
    match (dst, src) {
        (Value::Object(dst_map), Value::Object(src_map)) => {
            for (k, v) in src_map {
                deep_merge(dst_map.entry(k).or_insert(Value::Null), v);
            }
        }
        (dst_val, src_val) => {
            *dst_val = src_val;
        }
    }
}

/// Read config. Never creates one if missing — let OpenClaw own that.
pub async fn get_config() -> anyhow::Result<Value> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&path).await?;
    let cfg: Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("openclaw.json is malformed: {e}. Fix it manually or delete it."))?;
    Ok(cfg)
}

/// Write config with full safety:
/// 1. Read current config fresh (avoid stale in-memory state)
/// 2. Deep-merge changes (preserve nested keys the shell doesn't touch)
/// 3. Validate resulting JSON
/// 4. Atomically write to temp file, then rename
/// 5. Leave a .shell.bak copy
pub async fn set_config(incoming: Value) -> anyhow::Result<()> {
    let path = config_path()?;
    let bak = backup_path()?;

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 1. Read current fresh state
    let mut current = if path.exists() {
        let raw = fs::read_to_string(&path).await?;
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 2. Deep merge incoming over current
    deep_merge(&mut current, incoming);

    // 3. Validate: must round-trip through serde
    let pretty = serde_json::to_string_pretty(&current)?;
    let _: Value = serde_json::from_str(&pretty)?; // ensure valid

    // 4. Atomic write via temp + rename
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &pretty).await?;

    // 5. Backup old config before replacing
    if path.exists() {
        let _ = fs::copy(&path, &bak).await;
    }

    // 6. Atomically swap
    fs::rename(&tmp, &path).await?;

    Ok(())
}

/// Revert to the last shell backup. Use with caution.
pub async fn revert_config() -> anyhow::Result<()> {
    let path = config_path()?;
    let bak = backup_path()?;
    if bak.exists() {
        fs::copy(&bak, &path).await?;
        Ok(())
    } else {
        anyhow::bail!("No shell backup found")
    }
}
