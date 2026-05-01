use tauri::Manager;

use crate::services::settings_service::{self, AppSettings};

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    settings_service::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    settings_service::save_settings(&app, &settings)
}

#[tauri::command]
pub fn get_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;
    Ok(data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    Ok(())
}

const HOME_PLACEHOLDER: &str = "${HOME}";

fn home_dir_str() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn make_portable_path(path: &str, home: &str) -> String {
    normalize_path(path).replace(home, HOME_PLACEHOLDER)
}

fn resolve_portable_path(path: &str, home: &str) -> String {
    normalize_path(path).replace(HOME_PLACEHOLDER, home)
}

/// Export settings as JSON string with portable paths (${HOME} placeholders)
#[tauri::command]
pub fn export_config(settings: crate::services::settings_service::AppSettings) -> Result<String, String> {
    let mut portable = settings;
    let home = normalize_path(&home_dir_str());

    for agent in &mut portable.agents {
        agent.global_path = make_portable_path(&agent.global_path, &home);
        agent.project_path = make_portable_path(&agent.project_path, &home);
    }
    for repo in &mut portable.repos {
        repo.cache_path = make_portable_path(&repo.cache_path, &home);
    }

    serde_json::to_string_pretty(&portable)
        .map_err(|e| format!("Failed to serialize settings: {}", e))
}

/// Import settings from JSON string, resolving ${HOME} to actual paths
#[tauri::command]
pub fn import_config(
    app: tauri::AppHandle,
    json: String,
) -> Result<crate::services::settings_service::AppSettings, String> {
    let mut settings: crate::services::settings_service::AppSettings =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let home = normalize_path(&home_dir_str());

    for agent in &mut settings.agents {
        agent.global_path = resolve_portable_path(&agent.global_path, &home);
        agent.project_path = resolve_portable_path(&agent.project_path, &home);
    }
    for repo in &mut settings.repos {
        repo.cache_path = resolve_portable_path(&repo.cache_path, &home);
    }

    crate::services::settings_service::save_settings(&app, &settings)?;
    Ok(settings)
}
