use std::collections::HashMap;
use std::path::Path;

use crate::models::skills::*;
use crate::services::skills_service;

/// Get the user's home directory
fn dirs_home() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("~"))
}

/// Scan global skills for an agent
#[tauri::command]
pub async fn scan_global_skills(
    agent_id: String,
    global_path: String,
) -> Result<ScanResult, String> {
    // Expand ~ or relative paths
    let expanded = if global_path.starts_with("~/") {
        global_path.replacen("~", &dirs_home(), 1)
    } else {
        global_path
    };

    skills_service::scan_skills(&expanded, &agent_id, SkillScope::Global)
}

/// Scan project skills for an agent
#[tauri::command]
pub async fn scan_project_skills(
    agent_id: String,
    _project_path: String,
    project_dir: String,
) -> Result<ScanResult, String> {
    skills_service::scan_skills(&project_dir, &agent_id, SkillScope::Project)
}

/// Get file tree for a skill
#[tauri::command]
pub async fn get_skill_file_tree(skill_path: String) -> Result<Vec<FileEntry>, String> {
    skills_service::build_file_tree(&skill_path)
}

/// Read a skill file's content
#[tauri::command]
pub async fn read_skill_file(file_path: String) -> Result<String, String> {
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))
}

/// Calculate hash for a skill (file or directory)
#[tauri::command]
pub async fn calculate_skill_hash(skill_path: String) -> Result<String, String> {
    let path = Path::new(&skill_path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", skill_path));
    }

    if path.is_file() {
        let content = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        Ok(skills_service::calculate_hash(&content))
    } else {
        let mut file_count = 0u64;
        let mut total_size = 0u64;
        let mut hash_input = String::new();
        skills_service::collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;
        Ok(skills_service::calculate_hash(hash_input.as_bytes()))
    }
}

/// Get overview of skills across projects
#[tauri::command]
pub async fn get_projects_overview(
    project_paths: Vec<String>,
    agent_ids: Vec<String>,
    global_paths: HashMap<String, String>,
    project_patterns: HashMap<String, String>,
) -> Result<Vec<ProjectOverview>, String> {
    skills_service::get_projects_overview(
        &project_paths,
        &agent_ids,
        &global_paths,
        &project_patterns,
    )
}

/// Install a skill
#[tauri::command]
pub async fn install_skill(
    source: String,
    target_path: String,
) -> Result<OperationResult, String> {
    skills_service::install_skill(&source, &target_path)
}

/// Update a skill
#[tauri::command]
pub async fn update_skill(
    source: String,
    target_path: String,
) -> Result<OperationResult, String> {
    skills_service::update_skill(&source, &target_path)
}

/// Uninstall a skill
#[tauri::command]
pub async fn uninstall_skill(skill_path: String) -> Result<OperationResult, String> {
    skills_service::uninstall_skill(&skill_path)
}

/// Batch operate on multiple skills
#[tauri::command]
pub async fn batch_operate(
    operations: Vec<SkillOperation>,
) -> Vec<OperationResult> {
    skills_service::batch_operate(operations)
}

/// Verify skill integrity
#[tauri::command]
pub async fn verify_skill_integrity(
    skill_path: String,
    expected_hash: String,
) -> Result<bool, String> {
    skills_service::verify_skill_integrity(&skill_path, &expected_hash)
}
