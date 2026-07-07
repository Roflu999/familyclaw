use serde::Serialize;
use tokio::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct DebugInfo {
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub openclaw_version: Option<String>,
    pub openclaw_path: Option<String>,
    pub config_path: Option<String>,
    pub os: String,
    pub gateway_running: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct DoctorResult {
    pub raw_output: String,
    pub issues: Vec<DoctorIssue>,
    pub healthy: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct DoctorIssue {
    pub severity: String,
    pub message: String,
    pub fix_action: String,
    pub auto_fixable: bool,
}

pub async fn gather_info() -> anyhow::Result<DebugInfo> {
    let node = crate::installer::resolve_node();
    let npm = crate::installer::resolve_npm();
    let openclaw = crate::installer::resolve_openclaw();

    let node_version = if let Some(ref p) = node {
        let out = Command::new(p).arg("--version").output().await.ok();
        out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    let npm_version = if let Some(ref p) = npm {
        let out = Command::new(p).arg("--version").output().await.ok();
        out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    let openclaw_version = if let Some(ref p) = openclaw {
        let out = Command::new(p).arg("--version").output().await.ok();
        out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    let config_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json").to_string_lossy().to_string());

    let gateway = crate::openclaw::get_gateway_status().await.ok();

    Ok(DebugInfo {
        node_version,
        npm_version,
        openclaw_version,
        openclaw_path: openclaw.map(|p| p.to_string_lossy().to_string()),
        config_path,
        os: std::env::consts::OS.to_string(),
        gateway_running: gateway.map(|g| g.running).unwrap_or(false),
    })
}

/// Parse raw doctor output into plain-language actionable issues.
pub async fn parse_doctor(raw: String) -> DoctorResult {
    let mut issues = Vec::new();
    let lower = raw.to_lowercase();

    if lower.contains("node.js v") && lower.contains("is required") {
        issues.push(DoctorIssue {
            severity: "error".to_string(),
            message: "Node.js version is too old.".to_string(),
            fix_action: "The shell will auto-install the correct Node version.".to_string(),
            auto_fixable: true,
        });
    }

    if lower.contains("config") && (lower.contains("not found") || lower.contains("missing")) {
        issues.push(DoctorIssue {
            severity: "warning".to_string(),
            message: "OpenClaw config file is missing or incomplete.".to_string(),
            fix_action: "Run the setup wizard to create a fresh config.".to_string(),
            auto_fixable: true,
        });
    }

    // Fixed operator precedence: must be (port AND in_use) OR eaddrinuse
    if (lower.contains("port") && lower.contains("in use")) || lower.contains("eaddrinuse") {
        issues.push(DoctorIssue {
            severity: "error".to_string(),
            message: "Gateway port is already in use.".to_string(),
            fix_action: "The shell will auto-detect a free port.".to_string(),
            auto_fixable: true,
        });
    }

    if lower.contains("api key") && (lower.contains("not found") || lower.contains("missing") || lower.contains("invalid")) {
        issues.push(DoctorIssue {
            severity: "warning".to_string(),
            message: "No valid API key is configured.".to_string(),
            fix_action: "Go to API Keys and add a provider key.".to_string(),
            auto_fixable: false,
        });
    }

    if lower.contains("gateway") && lower.contains("not running") {
        issues.push(DoctorIssue {
            severity: "info".to_string(),
            message: "Gateway is not running.".to_string(),
            fix_action: "Click Start Gateway on the Dashboard.".to_string(),
            auto_fixable: true,
        });
    }

    if lower.contains("permission") || lower.contains("access denied") {
        issues.push(DoctorIssue {
            severity: "error".to_string(),
            message: "Permission denied — the app may need administrator rights.".to_string(),
            fix_action: "Run the shell as Administrator and try again.".to_string(),
            auto_fixable: false,
        });
    }

    if issues.is_empty() {
        issues.push(DoctorIssue {
            severity: "success".to_string(),
            message: "Everything looks good!".to_string(),
            fix_action: "No action needed.".to_string(),
            auto_fixable: true,
        });
    }

    let healthy = !issues.iter().any(|i| i.severity == "error");

    DoctorResult {
        raw_output: raw,
        issues,
        healthy,
    }
}

pub async fn open_folder() -> anyhow::Result<()> {
    let path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("home dir not found"))?
        .join(".openclaw");
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(&path).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(&path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(&path).spawn()?;
    }
    Ok(())
}
