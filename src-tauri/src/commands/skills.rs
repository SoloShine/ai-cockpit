use std::collections::HashMap;
use std::path::Path;

use crate::models::skills::*;
use crate::services::settings_service::RepoConfig;
use crate::services::{history_service, skills_service};

/// Get the user's home directory
fn dirs_home() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("~"))
}

/// Expand a path: ~ → home dir, relative → prepend home dir, absolute → as-is
fn expand_path(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }
    if path.starts_with('~') {
        return path.replacen("~", &dirs_home(), 1);
    }
    // Absolute path (Unix / or Windows C:\)
    if path.starts_with('/') || (path.len() >= 2 && path.as_bytes()[1] == b':') {
        return path.to_string();
    }
    // Relative path → prepend home directory
    format!("{}/{}", dirs_home(), path)
}

/// Scan global skills for an agent
#[tauri::command]
pub async fn scan_global_skills(
    agent_id: String,
    global_path: String,
) -> Result<ScanResult, String> {
    let expanded = expand_path(&global_path);
    skills_service::scan_skills(&expanded, &agent_id, SkillScope::Global)
}

/// Scan project skills for an agent
#[tauri::command]
pub async fn scan_project_skills(
    agent_id: String,
    project_path: String,
    project_dir: String,
) -> Result<ScanResult, String> {
    // project_dir = selected project root, project_path = agent's relative skill dir
    let full_path = if project_dir.is_empty() {
        return Ok(ScanResult {
            agent_id,
            scope: SkillScope::Project,
            skills: Vec::new(),
            total: 0,
        });
    } else {
        format!("{}/{}", project_dir, project_path)
    };
    skills_service::scan_skills(&full_path, &agent_id, SkillScope::Project)
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

/// Get rich overview of projects with comparison counts, README preview, and last-modified.
/// Scans remote repos only once for all projects.
#[tauri::command]
pub async fn get_rich_projects_overview(
    project_paths: Vec<String>,
    agent_id: String,
    project_pattern: String,
    repos: Vec<RepoConfig>,
) -> Result<Vec<ProjectOverview>, String> {
    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| (r.id.clone(), resolve_repo_cache(&r.cache_path, &r.id)))
        .collect();

    skills_service::get_rich_projects_overview(
        &project_paths,
        &agent_id,
        &project_pattern,
        &repo_dirs,
    )
}

/// Install a skill
#[tauri::command]
pub async fn install_skill(
    source: String,
    target_path: String,
) -> Result<OperationResult, String> {
    let target_path = expand_path(&target_path);
    let result = skills_service::install_skill(&source, &target_path)?;
    // Record in operation history
    let skill_name = Path::new(&target_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let _ = history_service::record_operation(
        history_service::OperationType::Install,
        skill_name,
        target_path,
        Some(source),
        None,
        None,
    );
    Ok(result)
}

/// Update a skill
#[tauri::command]
pub async fn update_skill(
    source: String,
    target_path: String,
) -> Result<OperationResult, String> {
    let target_path = expand_path(&target_path);
    let skill_name = Path::new(&target_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let result = skills_service::update_skill(&source, &target_path)?;
    let _ = history_service::record_operation(
        history_service::OperationType::Update,
        skill_name,
        target_path,
        Some(source),
        None,
        None,
    );
    Ok(result)
}

/// Uninstall a skill
#[tauri::command]
pub async fn uninstall_skill(skill_path: String) -> Result<OperationResult, String> {
    let skill_path = expand_path(&skill_path);
    let skill_name = Path::new(&skill_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let result = skills_service::uninstall_skill(&skill_path)?;
    let _ = history_service::record_operation(
        history_service::OperationType::Uninstall,
        skill_name,
        skill_path,
        None,
        None,
        None,
    );
    Ok(result)
}

/// Batch operate on multiple skills
#[tauri::command]
pub async fn batch_operate(
    mut operations: Vec<SkillOperation>,
) -> Vec<OperationResult> {
    // Expand ~ in target paths
    for op in &mut operations {
        op.target_path = expand_path(&op.target_path);
    }
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

/// Resolve a repo's cache path to an absolute directory.
fn resolve_repo_cache(cache_path: &str, repo_id: &str) -> String {
    if cache_path.is_empty() {
        dirs::data_dir()
            .map(|d| d.join("ai-cockpit").join("repos").join(repo_id))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    } else if cache_path.starts_with('~') {
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        cache_path.replacen("~", &home, 1)
    } else if cache_path.starts_with('/') || (cache_path.len() >= 2 && cache_path.as_bytes()[1] == b':') {
        // Already absolute (Unix or Windows)
        cache_path.to_string()
    } else {
        // Relative path → resolve under app data dir
        dirs::data_dir()
            .map(|d| d.join("ai-cockpit").join(cache_path))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| cache_path.to_string())
    }
}

/// Compare local skills against remote repos.
/// `local_dir` is the fully resolved skill directory (e.g. ~/.claude/skills).
/// Frontend is responsible for constructing it correctly.
#[tauri::command]
pub async fn compare_skills(
    local_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<Vec<SkillComparison>, String> {
    let expanded = expand_path(&local_dir);

    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| (r.id.clone(), resolve_repo_cache(&r.cache_path, &r.id)))
        .collect();

    skills_service::build_skill_comparisons(&expanded, &repo_dirs)
}

/// Get file-level diff between local and remote versions of a skill.
#[tauri::command]
pub async fn get_skill_diff(
    local_skill_path: String,
    remote_skill_path: String,
) -> Result<SkillDiffResult, String> {
    skills_service::build_skill_diff(&local_skill_path, &remote_skill_path)
}

/// Get content of a file from both local and remote skill for line diff.
#[tauri::command]
pub async fn get_diff_file_content(
    local_skill_path: String,
    remote_skill_path: String,
    rel_file_path: String,
) -> Result<DiffFileContent, String> {
    skills_service::get_diff_file_content(
        &local_skill_path,
        &remote_skill_path,
        &rel_file_path,
    )
}

/// Scan source and target skill directories to determine migration status of each skill.
#[tauri::command]
pub async fn scan_migrate_skills(
    source_path: String,
    target_path: String,
) -> Result<Vec<MigrateSkillItem>, String> {
    let expanded_source = expand_path(&source_path);
    let expanded_target = expand_path(&target_path);

    let source_result = skills_service::scan_skills(&expanded_source, "source", SkillScope::Global)?;
    let target_result = skills_service::scan_skills(&expanded_target, "target", SkillScope::Global)?;

    // Build a map of target skills by name
    let target_map: std::collections::HashMap<String, &SkillInfo> = target_result
        .skills
        .iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    let mut items = Vec::new();

    for source_skill in &source_result.skills {
        let target_skill_path = format!("{}/{}", expanded_target, source_skill.name);

        if let Some(target_skill) = target_map.get(&source_skill.name) {
            // Target exists — compare
            let status = if source_skill.content_hash == target_skill.content_hash {
                "sameContent".to_string()
            } else {
                // Check if versions differ
                let source_version = source_skill.meta.as_ref().and_then(|m| m.version.as_deref());
                let target_version = target_skill.meta.as_ref().and_then(|m| m.version.as_deref());

                if source_version != target_version {
                    "differentVersion".to_string()
                } else {
                    "contentDiffers".to_string()
                }
            };

            items.push(MigrateSkillItem {
                name: source_skill.name.clone(),
                source_path: source_skill.path.clone(),
                target_path: target_skill.path.clone(),
                status,
                source_hash: Some(source_skill.content_hash.clone()),
                target_hash: Some(target_skill.content_hash.clone()),
                version: source_skill.meta.as_ref().and_then(|m| m.version.clone()),
                description: source_skill.meta.as_ref().map(|m| m.description.clone()),
            });
        } else {
            // No target — new skill
            items.push(MigrateSkillItem {
                name: source_skill.name.clone(),
                source_path: source_skill.path.clone(),
                target_path: target_skill_path,
                status: "newTarget".to_string(),
                source_hash: Some(source_skill.content_hash.clone()),
                target_hash: None,
                version: source_skill.meta.as_ref().and_then(|m| m.version.clone()),
                description: source_skill.meta.as_ref().map(|m| m.description.clone()),
            });
        }
    }

    // Sort: contentDiffers first, then differentVersion, then newTarget, then sameContent
    items.sort_by(|a, b| {
        let order = |s: &str| match s {
            "contentDiffers" => 0,
            "differentVersion" => 1,
            "newTarget" => 2,
            "sameContent" => 3,
            _ => 4,
        };
        order(&a.status).cmp(&order(&b.status))
    });

    Ok(items)
}

/// Execute migration: copy selected skills from source to target agent directory.
#[tauri::command]
pub async fn migrate_skills(
    requests: Vec<MigrateRequest>,
) -> Result<MigrateResult, String> {
    let mut migrated = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();

    for req in &*requests {
        if req.resolution == "Skip" {
            skipped.push(req.name.clone());
            continue;
        }

        // resolution == "Overwrite" — remove target if exists, then copy
        let target = Path::new(&req.target_path);
        if target.exists() {
            if let Err(e) = skills_service::uninstall_skill(&req.target_path) {
                failed.push(MigrateFailedItem {
                    name: req.name.clone(),
                    error: format!("Failed to remove existing target: {}", e),
                });
                continue;
            }
        }

        match skills_service::install_skill(&req.source_path, &req.target_path) {
            Ok(_) => {
                // Record in history
                let _ = history_service::record_operation(
                    history_service::OperationType::Install,
                    req.name.clone(),
                    req.target_path.clone(),
                    Some(req.source_path.clone()),
                    None,
                    None,
                );
                migrated.push(req.name.clone());
            }
            Err(e) => {
                failed.push(MigrateFailedItem {
                    name: req.name.clone(),
                    error: e,
                });
            }
        }
    }

    Ok(MigrateResult {
        migrated,
        skipped,
        failed,
    })
}

// --- Skillbase dependency management ---

/// Resolve skillbase.json dependencies for a project
#[tauri::command]
pub async fn get_skillbase_resolution(
    skill_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<SkillbaseResolution, String> {
    let expanded = expand_path(&skill_dir);
    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| (r.id.clone(), resolve_repo_cache(&r.cache_path, &r.id)))
        .collect();

    skills_service::resolve_skillbase(&expanded, &repo_dirs)
}

/// Sync missing/mismatched/outdated skillbase dependencies
#[tauri::command]
pub async fn sync_skillbase_dependencies(
    skill_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<Vec<SkillbaseSyncResult>, String> {
    let expanded = expand_path(&skill_dir);
    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| (r.id.clone(), resolve_repo_cache(&r.cache_path, &r.id)))
        .collect();

    skills_service::sync_skillbase(&expanded, &repo_dirs)
}

/// Generate skillbase.json from currently installed skills
#[tauri::command]
pub async fn generate_skillbase_json(
    skill_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<String, String> {
    let expanded = expand_path(&skill_dir);
    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| (r.id.clone(), resolve_repo_cache(&r.cache_path, &r.id)))
        .collect();

    skills_service::generate_skillbase(&expanded, &repo_dirs)
}

/// Write skillbase.json content to project root
#[tauri::command]
pub async fn write_skillbase_json(
    project_path: String,
    content: String,
) -> Result<(), String> {
    let expanded = expand_path(&project_path);
    skills_service::write_skillbase(&expanded, &content)
}
