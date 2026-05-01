use crate::services::settings_service::{self, AppSettings};

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    settings_service::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    settings_service::save_settings(&app, &settings)
}
