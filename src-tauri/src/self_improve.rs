use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

// ─── Data Models ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lesson {
    pub id: String,
    pub timestamp: String,
    pub category: String,
    pub trigger: String,
    pub observation: String,
    pub correction: String,
    pub source: String,
    pub applied_count: u32,
    pub success_count: u32,
    pub auto_solved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorPattern {
    pub pattern: String,
    pub category: String,
    pub frequency: u32,
    pub last_seen: String,
    pub suggested_fix: String,
    pub lesson_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserPreference {
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct LessonStore {
    lessons: Vec<Lesson>,
    patterns: Vec<ErrorPattern>,
    preferences: Vec<UserPreference>,
    version: u32,
}

// ─── Store Paths ────────────────────────────────────────────────────────────

fn store_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    Ok(home.join(".openclaw").join(".shell-lessons.json"))
}

fn openclaw_learnings_dir() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    Ok(home.join(".openclaw").join("workspace").join(".learnings"))
}

async fn load_store() -> anyhow::Result<LessonStore> {
    let path = store_path()?;
    if !path.exists() {
        return Ok(LessonStore {
            version: 1,
            ..Default::default()
        });
    }
    let raw = fs::read_to_string(&path).await?;
    let store: LessonStore = serde_json::from_str(&raw)
        .unwrap_or_else(|_| LessonStore { version: 1, ..Default::default() });
    Ok(store)
}

async fn save_store(store: &LessonStore) -> anyhow::Result<()> {
    let path = store_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let tmp = path.with_extension("tmp");
    let pretty = serde_json::to_string_pretty(store)?;
    fs::write(&tmp, &pretty).await?;
    fs::rename(&tmp, &path).await?;
    Ok(())
}

// ─── Core API ───────────────────────────────────────────────────────────────

pub async fn capture_lesson(
    category: &str,
    trigger: &str,
    observation: &str,
    correction: &str,
    source: &str,
) -> anyhow::Result<Lesson> {
    let mut store = load_store().await?;

    let lesson = Lesson {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        category: category.to_string(),
        trigger: trigger.to_string(),
        observation: observation.to_string(),
        correction: correction.to_string(),
        source: source.to_string(),
        applied_count: 0,
        success_count: 0,
        auto_solved: false,
    };

    // Check if this is a recurrence of a known pattern
    let normalized = format!("{} {}", trigger, observation).to_lowercase();
    for pat in &mut store.patterns {
        if normalized.contains(&pat.pattern.to_lowercase()) {
            pat.frequency += 1;
            pat.last_seen = lesson.timestamp.clone();
            // If it's happened 3+ times and has a fix, mark auto-solvable
            if pat.frequency >= 3 {
                if let Some(l) = store.lessons.iter_mut().find(|l| l.id == pat.lesson_id) {
                    l.auto_solved = true;
                }
            }
        }
    }

    store.lessons.push(lesson.clone());

    // Prune: keep max 500 lessons, 100 patterns
    if store.lessons.len() > 500 {
        store.lessons.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        store.lessons.truncate(500);
    }
    if store.patterns.len() > 100 {
        store.patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        store.patterns.truncate(100);
    }

    save_store(&store).await?;

    // Also sync to OpenClaw's learnings directory
    let _ = sync_lesson_to_openclaw(&lesson).await;

    Ok(lesson)
}

pub async fn record_error(err: &anyhow::Error, source: &str) -> anyhow::Result<Lesson> {
    let text = err.to_string();
    let (category, observation, correction) = classify_error(&text);

    capture_lesson(
        &category,
        &text,
        &observation,
        &correction,
        source,
    ).await
}

fn classify_error(text: &str) -> (String, String, String) {
    let lower = text.to_lowercase();

    if lower.contains("port") && (lower.contains("in use") || lower.contains("eaddrinuse")) {
        return (
            "gateway".to_string(),
            "Gateway port is already occupied by another process.".to_string(),
            "Auto-detect a free port and persist it in config.".to_string(),
        );
    }

    if lower.contains("permission") || lower.contains("access denied") {
        return (
            "system".to_string(),
            "Insufficient permissions to perform the operation.".to_string(),
            "Run the shell as Administrator or check file permissions.".to_string(),
        );
    }

    if lower.contains("node") && (lower.contains("not found") || lower.contains("command not found")) {
        return (
            "install".to_string(),
            "Node.js runtime is missing or not in PATH.".to_string(),
            "Install the managed Node.js runtime via the setup wizard.".to_string(),
        );
    }

    if lower.contains("openclaw") && lower.contains("not found") {
        return (
            "install".to_string(),
            "OpenClaw CLI is not installed.".to_string(),
            "Install OpenClaw into the managed runtime directory.".to_string(),
        );
    }

    if lower.contains("config") && (lower.contains("malformed") || lower.contains("invalid")) {
        return (
            "config".to_string(),
            "Configuration file is corrupted or has invalid JSON.".to_string(),
            "Restore from the shell's automatic backup or re-run setup.".to_string(),
        );
    }

    if lower.contains("api key") || lower.contains("unauthorized") || lower.contains("auth") {
        return (
            "api".to_string(),
            "API key is missing, invalid, or expired.".to_string(),
            "Verify the key format and re-enter it in API Keys.".to_string(),
        );
    }

    if lower.contains("timeout") || lower.contains("timed out") {
        return (
            "network".to_string(),
            "Network request timed out.".to_string(),
            "Check internet connection or increase timeout in settings.".to_string(),
        );
    }

    (
        "general".to_string(),
        text.to_string(),
        "Review the error details and try again.".to_string(),
    )
}

pub async fn get_lessons(category: Option<String>, limit: usize) -> anyhow::Result<Vec<Lesson>> {
    let store = load_store().await?;
    let mut lessons = store.lessons;

    if let Some(cat) = category {
        lessons.retain(|l| l.category == cat);
    }

    lessons.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    lessons.truncate(limit);
    Ok(lessons)
}

pub async fn get_recurring_issues() -> anyhow::Result<Vec<ErrorPattern>> {
    let store = load_store().await?;
    let mut patterns: Vec<_> = store.patterns.into_iter()
        .filter(|p| p.frequency >= 2)
        .collect();
    patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
    Ok(patterns)
}

pub async fn match_pattern(text: &str) -> anyhow::Result<Vec<(Lesson, u32)>> {
    let store = load_store().await?;
    let lower = text.to_lowercase();
    let mut matches = Vec::new();

    for pat in &store.patterns {
        if lower.contains(&pat.pattern.to_lowercase()) {
            if let Some(lesson) = store.lessons.iter().find(|l| l.id == pat.lesson_id) {
                matches.push((lesson.clone(), pat.frequency));
            }
        }
    }

    // Also do fuzzy match on recent lessons
    for lesson in &store.lessons {
        let lesson_text = format!("{} {}", lesson.trigger, lesson.observation).to_lowercase();
        if lower.contains(&lesson.trigger.to_lowercase()) || lower.contains(&lesson.observation.to_lowercase()) {
            if !matches.iter().any(|(l, _)| l.id == lesson.id) {
                matches.push((lesson.clone(), 1));
            }
        }
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(matches)
}

pub async fn record_success(lesson_id: &str) -> anyhow::Result<()> {
    let mut store = load_store().await?;
    if let Some(lesson) = store.lessons.iter_mut().find(|l| l.id == lesson_id) {
        lesson.applied_count += 1;
        lesson.success_count += 1;
    }
    save_store(&store).await?;
    Ok(())
}

pub async fn record_failure(lesson_id: &str) -> anyhow::Result<()> {
    let mut store = load_store().await?;
    if let Some(lesson) = store.lessons.iter_mut().find(|l| l.id == lesson_id) {
        lesson.applied_count += 1;
        lesson.auto_solved = false; // Reset auto-solved if it failed again
    }
    save_store(&store).await?;
    Ok(())
}

// ─── User Preferences ───────────────────────────────────────────────────────

pub async fn record_preference(key: &str, value: &str, category: &str) -> anyhow::Result<()> {
    let mut store = load_store().await?;
    let now = chrono::Local::now().to_rfc3339();

    if let Some(pref) = store.preferences.iter_mut().find(|p| p.key == key) {
        pref.value = value.to_string();
        pref.confidence = (pref.confidence + 0.1).min(1.0);
        pref.updated_at = now;
    } else {
        store.preferences.push(UserPreference {
            key: key.to_string(),
            value: value.to_string(),
            confidence: 0.3, // Start low, grow with reaffirmations
            category: category.to_string(),
            created_at: now.clone(),
            updated_at: now,
        });
    }

    save_store(&store).await?;
    Ok(())
}

pub async fn get_preferences(category: Option<String>) -> anyhow::Result<Vec<UserPreference>> {
    let store = load_store().await?;
    let mut prefs = store.preferences;

    if let Some(cat) = category {
        prefs.retain(|p| p.category == cat);
    }

    prefs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    Ok(prefs)
}

pub async fn get_preference(key: &str) -> anyhow::Result<Option<UserPreference>> {
    let store = load_store().await?;
    Ok(store.preferences.into_iter().find(|p| p.key == key))
}

// ─── Insights ───────────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct Insight {
    pub kind: String,
    pub message: String,
    pub confidence: f32,
    pub action: Option<String>,
}

pub async fn generate_insights() -> anyhow::Result<Vec<Insight>> {
    let store = load_store().await?;
    let mut insights = Vec::new();

    // 1. Recurring error insight
    for pat in &store.patterns {
        if pat.frequency >= 3 {
            insights.push(Insight {
                kind: "recurring_error".to_string(),
                message: format!(
                    "'{}' has happened {} times. Consider applying the permanent fix.",
                    pat.pattern, pat.frequency
                ),
                confidence: (pat.frequency as f32 * 0.15).min(0.95),
                action: Some(pat.suggested_fix.clone()),
            });
        }
    }

    // 2. User preference insight
    let prefs = &store.preferences;
    for pref in prefs.iter().filter(|p| p.confidence > 0.5) {
        insights.push(Insight {
            kind: "preference".to_string(),
            message: format!("You consistently prefer {} = {}.", pref.key, pref.value),
            confidence: pref.confidence,
            action: None,
        });
    }

    // 3. Gateway stability insight
    let gateway_lessons: Vec<_> = store.lessons.iter().filter(|l| l.category == "gateway").collect();
    let recent_crashes = gateway_lessons.iter().filter(|l| {
        chrono::DateTime::parse_from_rfc3339(&l.timestamp)
            .map(|dt| dt > chrono::Local::now() - chrono::Duration::hours(24))
            .unwrap_or(false)
    }).count();

    if recent_crashes >= 2 {
        insights.push(Insight {
            kind: "stability".to_string(),
            message: format!("Gateway crashed {} times in the last 24 hours.", recent_crashes),
            confidence: 0.9,
            action: Some("Run Doctor and check the Debug panel for details.".to_string()),
        });
    }

    // 4. API key insight
    let api_errors = store.lessons.iter().filter(|l| l.category == "api").count();
    if api_errors >= 2 {
        insights.push(Insight {
            kind: "api_health".to_string(),
            message: "Multiple API key issues detected. Verify your keys are active and have credits.".to_string(),
            confidence: 0.8,
            action: Some("Go to API Keys and re-verify your providers.".to_string()),
        });
    }

    // 5. Family mode override insight
    if let Some(pref) = prefs.iter().find(|p| p.key == "family_mode_override") {
        if pref.confidence > 0.5 {
            insights.push(Insight {
                kind: "safety".to_string(),
                message: "You often disable family mode. If this is a personal machine, consider setting the default to off.".to_string(),
                confidence: pref.confidence,
                action: Some("Go to Safety settings to change the default.".to_string()),
            });
        }
    }

    insights.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    Ok(insights)
}

// ─── OpenClaw Sync ──────────────────────────────────────────────────────────

async fn sync_lesson_to_openclaw(lesson: &Lesson) -> anyhow::Result<()> {
    let dir = openclaw_learnings_dir()?;
    fs::create_dir_all(&dir).await?;

    let date = lesson.timestamp.split('T').next().unwrap_or("unknown");
    let path = dir.join(format!("shell-{}.md", date));

    let entry = format!(
        "\n## [{}] {} (from shell)\n\n**Trigger:** {}\n\n**Observation:** {}\n\n**Correction:** {}\n\n**Source:** {}\n\n**Auto-solved:** {}\n\n---\n",
        &lesson.timestamp[..19],
        lesson.category,
        lesson.trigger,
        lesson.observation,
        lesson.correction,
        lesson.source,
        if lesson.auto_solved { "Yes" } else { "No" }
    );

    let mut existing = if path.exists() {
        fs::read_to_string(&path).await?
    } else {
        format!("# Shell Learnings \u2014 {}\n\n", date)
    };

    existing.push_str(&entry);
    fs::write(&path, existing).await?;
    Ok(())
}

pub async fn sync_from_openclaw() -> anyhow::Result<u32> {
    let dir = openclaw_learnings_dir()?;
    if !dir.exists() {
        return Ok(0);
    }

    let mut store = load_store().await?;
    let mut imported = 0u32;

    let mut entries = fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("shell-") || !name.ends_with(".md") {
                continue; // Skip our own exports for now
            }
        }

        let content = fs::read_to_string(&path).await?;
        // Very naive parser: look for ## headers with trigger/observation/correction
        for block in content.split("---") {
            if block.contains("Trigger:") && block.contains("Observation:") {
                let trigger = extract_md_field(block, "Trigger").unwrap_or_default();
                let observation = extract_md_field(block, "Observation").unwrap_or_default();
                let correction = extract_md_field(block, "Correction").unwrap_or_default();

                if !trigger.is_empty() && !observation.is_empty() {
                    let lesson = Lesson {
                        id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Local::now().to_rfc3339(),
                        category: "openclaw".to_string(),
                        trigger,
                        observation,
                        correction,
                        source: "openclaw-sync".to_string(),
                        applied_count: 0,
                        success_count: 0,
                        auto_solved: false,
                    };
                    store.lessons.push(lesson);
                    imported += 1;
                }
            }
        }
    }

    if imported > 0 {
        save_store(&store).await?;
    }

    Ok(imported)
}

fn extract_md_field(block: &str, field: &str) -> Option<String> {
    let marker = format!("**{}:**", field);
    block.lines()
        .find(|l| l.trim().starts_with(&marker))
        .map(|l| l[marker.len()..].trim().to_string())
}

// ─── Auto-Capture Helpers for Integration ───────────────────────────────────

/// Wrap a result: on error, auto-capture a lesson and return the error.
pub async fn capture_on_error<T>(
    result: anyhow::Result<T>,
    source: &str,
) -> anyhow::Result<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let _ = record_error(&e, source).await;
            Err(e)
        }
    }
}

/// Observe a user action and learn from it.
pub async fn observe_user_action(
    action: &str,
    context: &str,
    outcome: &str,
) -> anyhow::Result<()> {
    let mut store = load_store().await?;

    // Learn preferences from action patterns
    if action == "set_api_key" {
        if let Some(provider) = context.split(':').next() {
            let _ = record_preference("preferred_provider", provider, "behavior").await;
        }
    }

    if action == "set_safety" {
        if context.contains("family_mode=false") {
            let _ = record_preference("family_mode_override", "true", "safety").await;
        }
    }

    if action == "gateway_start" && outcome == "success" {
        if let Some(port) = context.strip_prefix("port=") {
            let _ = record_preference("preferred_port", port, "gateway").await;
        }
    }

    // If outcome was a failure, capture as lesson
    if outcome.starts_with("error:") {
        let err_text = outcome.strip_prefix("error:").unwrap_or(outcome);
        let (cat, obs, fix) = classify_error(err_text);
        let _ = capture_lesson(&cat, action, &obs, &fix, "observe_user_action").await;
    }

    // Update patterns
    let normalized = format!("{} {}", action, outcome).to_lowercase();
    if let Some(pat) = store.patterns.iter_mut().find(|p| normalized.contains(&p.pattern.to_lowercase())) {
        pat.frequency += 1;
        pat.last_seen = chrono::Local::now().to_rfc3339();
    } else if !outcome.starts_with("success") {
        // New failure pattern
        store.patterns.push(ErrorPattern {
            pattern: action.to_string(),
            category: "behavior".to_string(),
            frequency: 1,
            last_seen: chrono::Local::now().to_rfc3339(),
            suggested_fix: "Review the action context.".to_string(),
            lesson_id: store.lessons.last().map(|l| l.id.clone()).unwrap_or_default(),
        });
    }

    save_store(&store).await?;
    Ok(())
}
