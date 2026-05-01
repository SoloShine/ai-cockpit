mod commands;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::core::get_app_version,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::get_data_dir,
            commands::settings::open_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AI Cockpit");
}
