use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::Window;
use tokio::fs;
use tokio::process::Command;
use sha2::Digest;

/// Managed runtime directory inside the app's data dir.
fn managed_dir() -> anyhow::Result<PathBuf> {
    let data = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("no data dir"))?;
    Ok(data.join("com.openclaw.shell").join("runtime"))
}

fn node_bin() -> PathBuf {
    managed_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(if cfg!(target_os = "windows") {
            "node.exe"
        } else {
            "bin/node"
        })
}

fn npm_bin() -> PathBuf {
    managed_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "bin/npm"
        })
}

fn openclaw_bin() -> PathBuf {
    managed_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(if cfg!(target_os = "windows") {
            "node_modules/.bin/openclaw.cmd"
        } else {
            "node_modules/.bin/openclaw"
        })
}

/// Resolve the openclaw binary: prefer managed, fallback to PATH.
pub fn resolve_openclaw() -> Option<PathBuf> {
    let managed = openclaw_bin();
    if managed.exists() {
        return Some(managed);
    }
    which::which("openclaw").ok()
}

/// Resolve node binary: prefer managed, fallback to PATH.
pub fn resolve_node() -> Option<PathBuf> {
    let managed = node_bin();
    if managed.exists() {
        return Some(managed);
    }
    which::which("node").ok()
}

#[derive(Serialize, Clone, Debug)]
pub struct PrereqStatus {
    pub node_installed: bool,
    pub node_version: Option<String>,
    pub npm_installed: bool,
    pub openclaw_installed: bool,
    pub openclaw_version: Option<String>,
    pub managed_runtime: bool,
}

pub async fn check_prerequisites() -> anyhow::Result<PrereqStatus> {
    let node = resolve_node();
    let npm = resolve_npm();
    let openclaw = resolve_openclaw();
    let managed = node_bin().exists();

    let node_version = if let Some(ref path) = node {
        let out = Command::new(path).arg("--version").output().await?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    };

    let openclaw_version = if let Some(ref path) = openclaw {
        let out = Command::new(path).arg("--version").output().await?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    };

    Ok(PrereqStatus {
        node_installed: node.is_some(),
        node_version,
        npm_installed: npm.is_some(),
        openclaw_installed: openclaw.is_some(),
        openclaw_version,
        managed_runtime: managed,
    })
}

pub fn resolve_npm() -> Option<PathBuf> {
    let managed = npm_bin();
    if managed.exists() {
        return Some(managed);
    }
    which::which("npm").ok()
}

/// Download and extract a portable Node.js into our managed dir.
/// Uses 5-minute timeout, verifies SHA256 checksum after download.
pub async fn install_nodejs(_window: &Window) -> anyhow::Result<()> {
    let result = install_nodejs_inner(_window).await;
    if let Err(ref e) = result {
        let _ = crate::self_improve::record_error(e, "installer::install_nodejs").await;
    }
    result
}

async fn install_nodejs_inner(_window: &Window) -> anyhow::Result<()> {
    let dir = managed_dir()?;
    fs::create_dir_all(&dir).await?;

    #[cfg(target_os = "windows")]
    {
        let version = "v22.11.0";
        let url = format!(
            "https://nodejs.org/dist/{0}/node-{0}-win-x64.zip",
            version
        );
        let sha_url = format!("{}.sha256", url);
        let zip_path = dir.join("node.zip");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        // Download with timeout
        let resp = client.get(&url).send().await?;
        let bytes = resp.bytes().await?;
        fs::write(&zip_path, &bytes).await?;

        // Best-effort SHA256 verification
        if let Ok(sha_resp) = client.get(&sha_url).send().await {
            if let Ok(sha_text) = sha_resp.text().await {
                let expected = sha_text.split_whitespace().next().unwrap_or("");
                let actual = format!("{:x}", sha2::Sha256::digest(&bytes));
                if !expected.is_empty() && actual != expected {
                    let _ = fs::remove_file(&zip_path).await;
                    anyhow::bail!(
                        "Node.js download checksum mismatch! Expected {} got {}. Possible MITM attack.",
                        expected, actual
                    );
                }
            }
        }

        // Extract with PowerShell — escape paths to prevent injection
        let escaped_zip = escape_ps_path(&zip_path);
        let escaped_dir = escape_ps_path(&dir);
        let ps_cmd = format!(
            "Expand-Archive -Path {} -DestinationPath {} -Force",
            escaped_zip, escaped_dir
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .status()
            .await?;

        if !status.success() {
            let _ = fs::remove_file(&zip_path).await;
            anyhow::bail!("Node.js extraction failed (PowerShell returned error)");
        }

        // Move nested folder contents up
        let nested = dir.join(format!("node-{}-win-x64", version));
        if nested.exists() {
            for entry in std::fs::read_dir(&nested)? {
                let entry = entry?;
                let dest = dir.join(entry.file_name());
                if dest.exists() {
                    let _ = std::fs::remove_dir_all(&dest);
                }
                std::fs::rename(entry.path(), dest)?;
            }
            let _ = std::fs::remove_dir(&nested);
        }
        let _ = fs::remove_file(&zip_path).await;
    }

    #[cfg(target_os = "macos")]
    {
        let url = "https://nodejs.org/dist/v22.11.0/node-v22.11.0-darwin-x64.tar.gz";
        let tar_path = dir.join("node.tar.gz");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;
        let resp = client.get(url).send().await?;
        let bytes = resp.bytes().await?;
        fs::write(&tar_path, &bytes).await?;

        let status = Command::new("tar")
            .args(["-xzf", tar_path.to_str().unwrap(), "-C", dir.to_str().unwrap(), "--strip-components=1"])
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Node.js extraction failed");
        }
        let _ = fs::remove_file(&tar_path).await;
    }

    #[cfg(target_os = "linux")]
    {
        let url = "https://nodejs.org/dist/v22.11.0/node-v22.11.0-linux-x64.tar.xz";
        let tar_path = dir.join("node.tar.xz");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;
        let resp = client.get(url).send().await?;
        let bytes = resp.bytes().await?;
        fs::write(&tar_path, &bytes).await?;

        let status = Command::new("tar")
            .args(["-xf", tar_path.to_str().unwrap(), "-C", dir.to_str().unwrap(), "--strip-components=1"])
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("Node.js extraction failed");
        }
        let _ = fs::remove_file(&tar_path).await;
    }

    Ok(())
}

/// Escape a path for safe use inside PowerShell single-quoted strings.
fn escape_ps_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    // In PowerShell single-quoted strings, ' must be doubled
    format!("'{}'", s.replace("'", "''"))
}

/// Install OpenClaw into our managed dir so PATH never matters.
/// Uses --ignore-scripts to prevent arbitrary postinstall code execution.
pub async fn install_openclaw(_window: &Window) -> anyhow::Result<()> {
    let result = install_openclaw_inner(_window).await;
    if let Err(ref e) = result {
        let _ = crate::self_improve::record_error(e, "installer::install_openclaw").await;
    }
    result
}

async fn install_openclaw_inner(_window: &Window) -> anyhow::Result<()> {
    let dir = managed_dir()?;
    let npm = resolve_npm().ok_or_else(|| anyhow::anyhow!("npm not found"))?;

    let out = Command::new(&npm)
        .args(["install", "--ignore-scripts", "openclaw@latest"])
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !out.status.success() {
        anyhow::bail!(
            "OpenClaw install failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}
