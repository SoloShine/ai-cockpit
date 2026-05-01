// src-tauri/src/commands/git_sync.rs
use crate::models::git_sync::*;
use crate::services::git_service;
use crate::services::settings_service::RepoConfig;
use crate::services::skills_service;

/// Resolve a repo's cache path to an absolute directory.
///
/// - Empty or no cachePath → auto-derive from repo URL hash under app data dir
/// - `~` prefix → expand to home directory
/// - Absolute path → use as-is
/// - Relative path → resolve under app data dir
fn resolve_cache_path(cache_path: &str, repo_id: &str) -> String {
    let app_data = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ai-cockpit")
        .join("repos");

    if cache_path.is_empty() {
        // Auto-derive from repo_id
        return app_data.join(repo_id).to_string_lossy().to_string();
    }

    let path = std::path::Path::new(cache_path);
    if cache_path.starts_with('~') {
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        return cache_path.replacen("~", &home, 1);
    }

    if path.is_absolute() {
        return cache_path.to_string();
    }

    // Relative path → resolve under app data dir
    app_data.join(cache_path).to_string_lossy().to_string()
}

/// Sync all enabled repositories in parallel.
/// Returns a SyncResult per repo indicating success/failure.
#[tauri::command]
pub async fn sync_all_repos(repos: Vec<RepoConfig>) -> Vec<SyncResult> {
    let enabled: Vec<_> = repos.into_iter().filter(|r| r.enabled).collect();
    if enabled.is_empty() {
        return vec![];
    }

    let handles: Vec<_> = enabled
        .into_iter()
        .map(|repo| {
            let repo_id = repo.id.clone();
            let expanded = resolve_cache_path(&repo.cache_path, &repo.id);
            let url = repo.url.clone();
            tokio::task::spawn_blocking(move || {
                let result = git_service::sync_repo(&url, &expanded);
                let skill_count = if result.is_ok() {
                    count_skills_in_dir(&expanded).unwrap_or(0)
                } else {
                    0
                };
                (repo_id, result, skill_count)
            })
        })
        .collect();

    // Collect results from all tasks
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
        .into_iter()
        .map(|(repo_id, result, skill_count)| match result {
            Ok(()) => SyncResult {
                repo_id,
                success: true,
                message: "Synced successfully".to_string(),
                skill_count,
            },
            Err(e) => SyncResult {
                repo_id,
                success: false,
                message: e,
                skill_count: 0,
            },
        })
        .collect()
}

/// Get all skills from a specific repository's cached directory.
#[tauri::command]
pub async fn get_remote_skills(
    repo_id: String,
    cache_path: String,
) -> Result<Vec<RemoteSkillInfo>, String> {
    let cache_path = resolve_cache_path(&cache_path, &repo_id);
    let path = std::path::Path::new(&cache_path);
    if !path.exists() {
        return Ok(vec![]);
    }

    let scan_result = skills_service::scan_remote_skills(&cache_path, &repo_id)?;

    Ok(scan_result
        .skills
        .into_iter()
        .map(|s| RemoteSkillInfo {
            name: s.name,
            description: s.meta.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
            version: s.meta.as_ref().and_then(|m| m.version.clone()),
            source_repo: repo_id.clone(),
            skill_type: if s.is_file { "file".to_string() } else { "directory".to_string() },
        })
        .collect())
}

/// Get detailed info about a specific remote skill.
#[tauri::command]
pub async fn get_remote_skill_detail(
    repo_id: String,
    cache_path: String,
    skill_name: String,
) -> Result<RemoteSkillDetail, String> {
    let cache_path = resolve_cache_path(&cache_path, &repo_id);
    let skill_path = std::path::Path::new(&cache_path).join(&skill_name);
    if !skill_path.exists() {
        return Err(format!("Skill '{}' not found in repo '{}'", skill_name, repo_id));
    }

    let files = skills_service::build_file_tree(skill_path.to_string_lossy().to_string().as_str())?;

    let scan_result = skills_service::scan_remote_skills(&cache_path, &repo_id)?;
    let skill_info = scan_result
        .skills
        .into_iter()
        .find(|s| s.name == skill_name)
        .ok_or_else(|| format!("Skill '{}' not found", skill_name))?;

    Ok(RemoteSkillDetail {
        info: RemoteSkillInfo {
            name: skill_info.name,
            description: skill_info.meta.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
            version: skill_info.meta.as_ref().and_then(|m| m.version.clone()),
            source_repo: repo_id,
            skill_type: if skill_info.is_file { "file".to_string() } else { "directory".to_string() },
        },
        files,
        content_hash: skill_info.content_hash,
    })
}

/// Count skill directories/files in a cached repo
fn count_skills_in_dir(cache_path: &str) -> Result<u32, String> {
    let result = skills_service::scan_remote_skills(cache_path, "")?;
    Ok(result.total as u32)
}
