use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;
use tokio::net::TcpListener;

#[derive(Serialize, Clone, Debug)]
pub struct GatewayStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub health: Option<String>,
    pub managed_by_shell: bool,
}

fn pid_file() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.openclaw.shell")
        .join("gateway.pid")
}

fn openclaw_cmd() -> anyhow::Result<Command> {
    let bin = crate::installer::resolve_openclaw()
        .ok_or_else(|| anyhow::anyhow!("openclaw not found"))?;
    Ok(Command::new(&bin))
}

/// Find a free port starting from the preferred one.
pub async fn find_free_port(preferred: u16) -> anyhow::Result<u16> {
    for port in preferred..=preferred + 100 {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => {
                let local = listener.local_addr()?;
                drop(listener);
                return Ok(local.port());
            }
            Err(_) => continue,
        }
    }
    anyhow::bail!("No free port found in range {}-{}", preferred, preferred + 100)
}

pub async fn get_gateway_status() -> anyhow::Result<GatewayStatus> {
    let running = is_gateway_running().await?;
    let port = detect_gateway_port().await;
    let shell_pid = read_shell_pid().await.ok().flatten();
    // Verify the PID file actually points to an openclaw process (protects against PID reuse)
    let managed = if let Some(pid) = shell_pid {
        running && is_openclaw_process(pid).await.unwrap_or(false)
    } else {
        false
    };

    let health = if running {
        if let Some(p) = port {
            check_health(p).await.ok()
        } else {
            None
        }
    } else {
        None
    };

    Ok(GatewayStatus {
        running,
        pid: shell_pid,
        port,
        health,
        managed_by_shell: managed,
    })
}

async fn is_gateway_running() -> anyhow::Result<bool> {
    #[cfg(target_os = "windows")]
    {
        // Use wmic to get command-line, more reliable than tasklist
        let out = Command::new("wmic")
            .args(["process", "where", "name='node.exe'", "get", "CommandLine", "/format:csv"])
            .output()
            .await?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        Ok(stdout.to_lowercase().contains("openclaw") && stdout.to_lowercase().contains("gateway"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let out = Command::new("pgrep")
            .args(["-f", "openclaw gateway"])
            .output()
            .await?;
        Ok(out.status.success())
    }
}

/// Verify that a given PID is actually an openclaw/node process.
async fn is_openclaw_process(pid: u32) -> anyhow::Result<bool> {
    #[cfg(target_os = "windows")]
    {
        let out = Command::new("wmic")
            .args(["process", "where", &format!("ProcessId={}", pid), "get", "CommandLine", "/format:csv"])
            .output()
            .await?;
        let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
        Ok(stdout.contains("openclaw") || stdout.contains("node"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let out = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .await?;
        let name = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
        Ok(name.contains("node") || name.contains("openclaw"))
    }
}

async fn read_shell_pid() -> anyhow::Result<Option<u32>> {
    let path = pid_file();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).await?;
    Ok(raw.trim().parse::<u32>().ok())
}

async fn write_shell_pid(pid: u32) -> anyhow::Result<()> {
    let path = pid_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&path, pid.to_string()).await?;
    Ok(())
}

async fn clear_shell_pid() -> anyhow::Result<()> {
    let path = pid_file();
    if path.exists() {
        fs::remove_file(&path).await?;
    }
    Ok(())
}

async fn detect_gateway_port() -> Option<u16> {
    if let Ok(cfg) = crate::config::get_config().await {
        if let Some(port) = cfg
            .get("gateway")
            .and_then(|g| g.get("port"))
            .and_then(|p| p.as_u64())
        {
            return Some(port as u16);
        }
    }
    Some(18789)
}

async fn check_health(port: u16) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await?;
    Ok(format!("HTTP {}", resp.status()))
}

pub async fn start_gateway() -> anyhow::Result<u32> {
    let result = start_gateway_inner().await;
    match &result {
        Ok(pid) => {
            let _ = crate::self_improve::observe_user_action("gateway_start", &format!("pid={}", pid), "success").await;
        }
        Err(e) => {
            let _ = crate::self_improve::record_error(e, "openclaw::start_gateway").await;
        }
    }
    result
}

async fn start_gateway_inner() -> anyhow::Result<u32> {
    // If already running independently, don't start another
    if is_gateway_running().await? {
        let existing = read_shell_pid().await.ok().flatten();
        if existing.is_none() {
            anyhow::bail!(
                "Gateway is already running (started outside the shell). \
                 Stop it first with 'openclaw gateway stop' or use the existing instance."
            );
        }
    }

    let mut cmd = openclaw_cmd()?;

    let free_port = find_free_port(18789).await?;
    if free_port != 18789 {
        // Only write port if we had to pick a non-default one
        let mut cfg = crate::config::get_config().await?;
        if !cfg.get("gateway").is_some() {
            cfg["gateway"] = serde_json::json!({});
        }
        cfg["gateway"]["port"] = serde_json::json!(free_port);
        crate::config::set_config(cfg.clone()).await?;
    }

    let child = cmd
        .args(["gateway", "start"])
        .env("OPENCLAW_GATEWAY_PORT", free_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let pid = child.id().unwrap_or(0);
    if pid == 0 {
        anyhow::bail!("Failed to obtain gateway process ID after spawning.");
    }
    write_shell_pid(pid).await?;

    // Verify the gateway actually bound to the port before declaring success
    let mut started = false;
    for attempt in 0..30 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if check_health(free_port).await.is_ok() {
            started = true;
            break;
        }
        // Also verify the process hasn't exited early
        if !is_process_running(pid).await {
            anyhow::bail!(
                "Gateway process exited before binding to port {} (attempt {}). Check logs for errors.",
                free_port, attempt
            );
        }
    }

    if !started {
        // Best-effort cleanup of the failed process
        let _ = stop_gateway_inner(pid).await;
        anyhow::bail!(
            "Gateway failed to start on port {} within 15 seconds. Check logs for errors.",
            free_port
        );
    }

    Ok(pid)
}

pub async fn stop_gateway(pid: u32) -> anyhow::Result<()> {
    let result = stop_gateway_inner(pid).await;
    if let Err(ref e) = result {
        let _ = crate::self_improve::record_error(e, "openclaw::stop_gateway").await;
    }
    result
}

async fn stop_gateway_inner(_pid: u32) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // First try graceful termination via taskkill (sends WM_CLOSE to console apps)
        let _ = Command::new("taskkill")
            .args(["/T", "/PID", &_pid.to_string()])
            .output()
            .await?;

        // Wait a moment
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Force kill if still running
        if is_process_running(_pid).await {
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &_pid.to_string()])
                .output()
                .await?;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Graceful SIGTERM first
        let _ = Command::new("kill")
            .args(["-TERM", &_pid.to_string()])
            .output()
            .await?;

        // Wait
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Force kill if still running
        if is_process_running(_pid).await {
            let _ = Command::new("kill")
                .args(["-KILL", &_pid.to_string()])
                .output()
                .await?;
        }
    }

    clear_shell_pid().await?;
    Ok(())
}

async fn is_process_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Use findstr to match exact PID at start of line in CSV output
        let out = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV"])
            .output()
            .await
            .ok();
        match out {
            Some(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                // CSV format: "Image Name","PID",...
                // Look for a line where the second field exactly matches our PID
                text.lines().any(|line| {
                    let cols: Vec<&str> = line.split(',').collect();
                    cols.get(1)
                        .map(|c| c.trim_matches('"') == pid.to_string())
                        .unwrap_or(false)
                })
            }
            None => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

pub async fn get_logs(lines: usize) -> anyhow::Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("home dir not found"))?;
    let log_dir = home.join(".openclaw").join("logs");
    if !log_dir.exists() {
        return Ok("No logs directory found.".to_string());
    }

    let mut entries = fs::read_dir(&log_dir).await?;
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let meta = entry.metadata().await?;
        // Skip symlinks to prevent arbitrary file reads
        if meta.is_symlink() {
            continue;
        }
        if meta.is_file() {
            let modified = meta.modified()?;
            if newest.as_ref().map(|(_, t)| modified > *t).unwrap_or(true) {
                newest = Some((path, modified));
            }
        }
    }

    if let Some((path, _)) = newest {
        let content = fs::read_to_string(&path).await?;
        let all: Vec<&str> = content.lines().collect();
        let start = all.len().saturating_sub(lines);
        Ok(all[start..].join("\n"))
    } else {
        Ok("No log files found.".to_string())
    }
}

pub async fn run_doctor() -> anyhow::Result<String> {
    let mut cmd = openclaw_cmd()?;
    let out = cmd.args(["doctor"]).output().await?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub async fn launch_tui() -> anyhow::Result<()> {
    let bin = crate::installer::resolve_openclaw()
        .ok_or_else(|| anyhow::anyhow!("openclaw not found"))?;
    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fall back to cmd
        let wt = Command::new("where")
            .arg("wt")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);
        if wt {
            Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg("wt")
                .arg(&bin)
                .arg("chat")
                .spawn()?;
        } else {
            Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg(&bin)
                .arg("chat")
                .spawn()?;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new(&bin);
        cmd.arg("chat").spawn()?;
    }
    Ok(())
}

pub async fn launch_dashboard() -> anyhow::Result<()> {
    let mut cmd = openclaw_cmd()?;
    cmd.args(["dashboard"]).spawn()?;
    Ok(())
}
