#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{Manager, State};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;

mod installer;
mod openclaw;
mod config;
mod api_keys;
mod safety;
mod debug;
mod backup;
mod channels;
mod skills;
mod self_improve;

struct AppState {
    gateway_pid: Mutex<Option<u32>>,
    gateway_managed: Mutex<bool>,
}

#[tauri::command]
async fn check_prerequisites() -> Result<installer::PrereqStatus, String> {
    installer::check_prerequisites().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_nodejs(window: tauri::Window) -> Result<(), String> {
    installer::install_nodejs(&window).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_openclaw(window: tauri::Window) -> Result<(), String> {
    installer::install_openclaw(&window).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn gateway_status(state: State<'_, AppState>) -> Result<openclaw::GatewayStatus, String> {
    let status = openclaw::get_gateway_status().await.map_err(|e| e.to_string())?;
    let mut managed = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
    *managed = status.managed_by_shell;
    Ok(status)
}

#[tauri::command]
async fn start_gateway(state: State<'_, AppState>) -> Result<u32, String> {
    let status = openclaw::get_gateway_status().await.map_err(|e| e.to_string())?;
    if status.running && !status.managed_by_shell {
        return Err("Gateway already running outside shell. Stop it via CLI first.".to_string());
    }
    // If already running and managed by us, just return the existing PID
    if status.running && status.managed_by_shell {
        if let Some(pid) = status.pid {
            let mut pid_guard = state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
            *pid_guard = Some(pid);
            let mut managed = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
            *managed = true;
            return Ok(pid);
        }
    }
    let pid = openclaw::start_gateway().await.map_err(|e| e.to_string())?;
    let mut pid_guard = state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
    *pid_guard = Some(pid);
    let mut managed = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
    *managed = true;
    Ok(pid)
}

#[tauri::command]
async fn stop_gateway(state: State<'_, AppState>) -> Result<(), String> {
    let (managed, pid) = {
        let mg = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
        let pg = state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
        (*mg, *pg)
    };
    if !managed {
        return Err("Gateway was not started by this shell. Use CLI to stop it.".to_string());
    }
    if let Some(pid) = pid {
        openclaw::stop_gateway(pid).await.map_err(|e| e.to_string())?;
    }
    {
        let mut pg = state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
        *pg = None;
        let mut mg = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
        *mg = false;
    }
    Ok(())
}

#[tauri::command]
async fn restart_gateway(state: State<'_, AppState>) -> Result<u32, String> {
    {
        let managed = *state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
        let pid = *state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
        if managed {
            if let Some(pid) = pid {
                let _ = openclaw::stop_gateway(pid).await;
            }
        }
    }
    let pid = openclaw::start_gateway().await.map_err(|e| e.to_string())?;
    let mut pg = state.gateway_pid.lock().unwrap_or_else(|e| e.into_inner());
    *pg = Some(pid);
    let mut mg = state.gateway_managed.lock().unwrap_or_else(|e| e.into_inner());
    *mg = true;
    Ok(pid)
}

#[tauri::command]
async fn get_logs(lines: usize) -> Result<String, String> {
    openclaw::get_logs(lines).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_doctor() -> Result<debug::DoctorResult, String> {
    let raw = openclaw::run_doctor().await.map_err(|e| e.to_string())?;
    Ok(debug::parse_doctor(raw).await)
}

#[tauri::command]
async fn get_openclaw_config() -> Result<serde_json::Value, String> {
    config::get_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_openclaw_config(cfg: serde_json::Value) -> Result<(), String> {
    config::set_config(cfg).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_api_keys() -> Result<Vec<api_keys::ApiKeyEntry>, String> {
    api_keys::list_keys().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_api_key(provider: String, key: String) -> Result<bool, String> {
    api_keys::set_key(&provider, &key).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_api_key(provider: String) -> Result<(), String> {
    api_keys::delete_key(&provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_safety_settings() -> Result<safety::SafetySettings, String> {
    safety::get_settings().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_safety_settings(settings: safety::SafetySettings) -> Result<(), String> {
    safety::set_settings(settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_backup() -> Result<String, String> {
    backup::create_backup().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn restore_backup(path: String) -> Result<(), String> {
    backup::restore_backup(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_tui() -> Result<(), String> {
    openclaw::launch_tui().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_dashboard() -> Result<(), String> {
    openclaw::launch_dashboard().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_debug_info() -> Result<debug::DebugInfo, String> {
    debug::gather_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_openclaw_folder() -> Result<(), String> {
    debug::open_folder().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_channel_guides() -> Result<Vec<channels::ChannelGuide>, String> {
    Ok(channels::get_channel_guides())
}

#[tauri::command]
async fn save_channel_config(channel: String, token: String) -> Result<(), String> {
    channels::save_channel_config(channel, token).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_skills() -> Result<Vec<skills::SkillReview>, String> {
    skills::list_skills().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_skill_enabled(name: String, enabled: bool) -> Result<(), String> {
    skills::set_skill_enabled(name, enabled).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_self_improve_lessons(category: Option<String>, limit: usize) -> Result<Vec<self_improve::Lesson>, String> {
    self_improve::get_lessons(category, limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_self_improve_insights() -> Result<Vec<self_improve::Insight>, String> {
    self_improve::generate_insights().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn match_error_pattern(text: String) -> Result<Vec<(self_improve::Lesson, u32)>, String> {
    self_improve::match_pattern(&text).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn record_user_preference(key: String, value: String, category: String) -> Result<(), String> {
    self_improve::record_preference(&key, &value, &category).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_user_preferences(category: Option<String>) -> Result<Vec<self_improve::UserPreference>, String> {
    self_improve::get_preferences(category).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_openclaw_learnings() -> Result<u32, String> {
    self_improve::sync_from_openclaw().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mark_lesson_success(lesson_id: String) -> Result<(), String> {
    self_improve::record_success(&lesson_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mark_lesson_failure(lesson_id: String) -> Result<(), String> {
    self_improve::record_failure(&lesson_id).await.map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            gateway_pid: Mutex::new(None),
            gateway_managed: Mutex::new(false),
        })
        .setup(|app| {
            // Build tray menu
            let menu = MenuBuilder::new(app)
                .item(&MenuItemBuilder::new("Show OpenClaw Shell").id("show").build(app)?)
                .item(&MenuItemBuilder::new("Open Dashboard").id("dashboard").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Start Gateway").id("start").build(app)?)
                .item(&MenuItemBuilder::new("Stop Gateway").id("stop").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Quit").id("quit").build(app)?)
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("OpenClaw Shell")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "dashboard" => {
                            let _ = tauri::async_runtime::block_on(openclaw::launch_dashboard());
                        }
                        "start" => {
                            let state = app.state::<AppState>();
                            let _ = tauri::async_runtime::block_on(async move {
                                let _ = start_gateway(state).await;
                            });
                        }
                        "stop" => {
                            let state = app.state::<AppState>();
                            let _ = tauri::async_runtime::block_on(async move {
                                let _ = stop_gateway(state).await;
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of quitting — let tray keep app alive
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            check_prerequisites,
            install_nodejs,
            install_openclaw,
            gateway_status,
            start_gateway,
            stop_gateway,
            restart_gateway,
            get_logs,
            run_doctor,
            get_openclaw_config,
            set_openclaw_config,
            list_api_keys,
            set_api_key,
            delete_api_key,
            get_safety_settings,
            set_safety_settings,
            create_backup,
            restore_backup,
            launch_tui,
            launch_dashboard,
            get_debug_info,
            open_openclaw_folder,
            get_channel_guides,
            save_channel_config,
            list_skills,
            set_skill_enabled,
            list_self_improve_lessons,
            get_self_improve_insights,
            match_error_pattern,
            record_user_preference,
            get_user_preferences,
            sync_openclaw_learnings,
            mark_lesson_success,
            mark_lesson_failure,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
