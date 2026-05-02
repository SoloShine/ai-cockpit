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
            // Core
            commands::core::get_app_version,
            // Settings
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::get_data_dir,
            commands::settings::open_in_explorer,
            commands::settings::export_config,
            commands::settings::import_config,
            // Git Sync
            commands::git_sync::sync_all_repos,
            commands::git_sync::get_remote_skills,
            commands::git_sync::get_remote_skill_detail,
            // Skills — Batch 1: Scan
            commands::skills::scan_global_skills,
            commands::skills::scan_project_skills,
            commands::skills::get_skill_file_tree,
            commands::skills::read_skill_file,
            commands::skills::calculate_skill_hash,
            commands::skills::get_projects_overview,
            // Skills — Batch 2: Operations
            commands::skills::install_skill,
            commands::skills::update_skill,
            commands::skills::uninstall_skill,
            commands::skills::batch_operate,
            commands::skills::verify_skill_integrity,
            // Skills — Batch 3: Comparison
            commands::skills::compare_skills,
            commands::skills::get_skill_diff,
            commands::skills::get_diff_file_content,
            // History
            commands::history::get_operation_history,
            commands::history::rollback_operation,
            commands::history::clear_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AI Cockpit");
}
