use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

use crate::models::skills::*;

/// Scan a directory for skills (files .md or directories with SKILL.md)
pub fn scan_skills(
    base_dir: &str,
    agent_id: &str,
    scope: SkillScope,
) -> Result<ScanResult, String> {
    let base = Path::new(base_dir);
    if !base.exists() {
        return Ok(ScanResult {
            agent_id: agent_id.to_string(),
            scope,
            skills: Vec::new(),
            total: 0,
        });
    }

    let mut skills = Vec::new();

    let entries = std::fs::read_dir(base)
        .map_err(|e| format!("Failed to read directory {}: {}", base_dir, e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        // Skip hidden files/directories
        if name.starts_with('.') {
            continue;
        }

        let skill_info = if path.is_file() {
            // Single .md file skill
            if name.ends_with(".md") {
                build_file_skill_info(&path, &name, agent_id)?
            } else {
                continue;
            }
        } else if path.is_dir() {
            // Directory skill (must contain SKILL.md)
            build_dir_skill_info(&path, &name, agent_id)?
        } else {
            continue;
        };

        skills.push(skill_info);
    }

    let total = skills.len() as u64;

    Ok(ScanResult {
        agent_id: agent_id.to_string(),
        scope,
        skills,
        total,
    })
}

/// Scan a remote repo for skills with two-strategy approach:
/// 1. Try parsing skills.json manifest first
/// 2. Fallback: scan root-level dirs + skills/ subdirectory for SKILL.md
pub fn scan_remote_skills(
    repo_path: &str,
    repo_id: &str,
) -> Result<ScanResult, String> {
    let base = Path::new(repo_path);
    if !base.exists() {
        return Ok(ScanResult {
            agent_id: repo_id.to_string(),
            scope: SkillScope::Global,
            skills: Vec::new(),
            total: 0,
        });
    }

    // Strategy 1: Try skills.json manifest
    let manifest_path = base.join("skills.json");
    if manifest_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(skills_list) = manifest.get("skills").and_then(|v| v.as_array()) {
                    let mut skills = Vec::new();
                    for entry in skills_list {
                        let skill_name = entry.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let skill_path = entry.get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        if skill_name.is_empty() {
                            continue;
                        }

                        let full_path = base.join(&skill_path);
                        let is_file = full_path.is_file();

                        let meta = if full_path.is_dir() {
                            let skill_md = full_path.join("SKILL.md");
                            if skill_md.exists() {
                                parse_skill_meta(&skill_md).ok().flatten()
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let (file_count, size_bytes, content_hash) = if full_path.exists() {
                            let mut fc = 0u64;
                            let mut sz = 0u64;
                            let mut hi = String::new();
                            if full_path.is_dir() {
                                let _ = collect_dir_stats(&full_path, &mut fc, &mut sz, &mut hi);
                            } else if full_path.is_file() {
                                if let Ok(content) = std::fs::read(&full_path) {
                                    fc = 1;
                                    sz = content.len() as u64;
                                    hi = calculate_hash(&content);
                                }
                            }
                            (fc, sz, calculate_hash(hi.as_bytes()))
                        } else {
                            (0, 0, String::new())
                        };

                        skills.push(SkillInfo {
                            name: skill_name,
                            path: full_path.to_string_lossy().to_string(),
                            is_file,
                            has_skill_md: full_path.join("SKILL.md").exists(),
                            meta,
                            file_count,
                            size_bytes,
                            content_hash,
                            last_modified: get_modified_time(&full_path),
                            source_agent_id: Some(repo_id.to_string()),
                        });
                    }
                    let total = skills.len() as u64;
                    return Ok(ScanResult {
                        agent_id: repo_id.to_string(),
                        scope: SkillScope::Global,
                        skills,
                        total,
                    });
                }
            }
        }
    }

    // Strategy 2: Scan directories for SKILL.md (root level + skills/ subdirectory)
    let mut scan_dirs: Vec<std::path::PathBuf> = Vec::new();

    // Root level subdirectories (skip "skills" itself to avoid double counting)
    if let Ok(rd) = std::fs::read_dir(base) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    if !name.starts_with('.') && name != "skills" {
                        scan_dirs.push(p);
                    }
                }
            }
        }
    }

    // skills/ subdirectory contents (e.g. anthropics/skills repo pattern)
    let skills_subdir = base.join("skills");
    if skills_subdir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&skills_subdir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Some(name) = e.file_name().to_str() {
                        if !name.starts_with('.') {
                            scan_dirs.push(p);
                        }
                    }
                }
            }
        }
    }

    let mut skills = Vec::new();
    for dir in &scan_dirs {
        let skill_md = dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        skills.push(build_dir_skill_info(dir, &name, repo_id)?);
    }

    let total = skills.len() as u64;
    Ok(ScanResult {
        agent_id: repo_id.to_string(),
        scope: SkillScope::Global,
        skills,
        total,
    })
}

/// Calculate SHA256 hash of content
pub fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Collect directory statistics and calculate aggregate hash
/// This must be pub because commands use it directly
pub fn collect_dir_stats(
    dir: &Path,
    file_count: &mut u64,
    total_size: &mut u64,
    hash_input: &mut String,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    let mut paths = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        paths.push(path);
    }

    // Sort for deterministic hash calculation
    paths.sort();

    for path in paths {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_file() {
            let content = std::fs::read(&path)
                .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
            let size = content.len() as u64;
            *file_count += 1;
            *total_size += size;

            let hash = calculate_hash(&content);
            hash_input.push_str(&format!("{}:{}\n", name, hash));
        } else if path.is_dir() {
            collect_dir_stats(&path, file_count, total_size, hash_input)?;
        }
    }

    Ok(())
}

/// Build file tree recursively
fn build_file_tree_recursive(path: &Path) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    let dir_entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?;

    let mut paths = Vec::new();
    for entry in dir_entries.flatten() {
        let p = entry.path();
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !name.starts_with('.') {
            paths.push((p, name));
        }
    }

    paths.sort_by(|a, b| a.1.cmp(&b.1));

    for (path, name) in paths {
        let metadata = path
            .metadata()
            .map_err(|e| format!("Failed to get metadata for {}: {}", path.display(), e))?;

        let size = metadata.len();
        let is_dir = path.is_dir();

        let children = if is_dir {
            build_file_tree_recursive(&path)?
        } else {
            Vec::new()
        };

        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            children,
        });
    }

    Ok(entries)
}

/// Build file tree for a skill
pub fn build_file_tree(path: &str) -> Result<Vec<FileEntry>, String> {
    let skill_path = Path::new(path);
    if !skill_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    build_file_tree_recursive(skill_path)
}

/// Build skill info for a file-based skill
fn build_file_skill_info(
    path: &Path,
    name: &str,
    agent_id: &str,
) -> Result<SkillInfo, String> {
    let content = std::fs::read(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let content_hash = calculate_hash(&content);

    let meta = parse_skill_meta_from_content(&content);

    let last_modified = get_modified_time(path);

    Ok(SkillInfo {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        is_file: true,
        has_skill_md: name.eq_ignore_ascii_case("SKILL.md"),
        meta,
        file_count: 1,
        size_bytes: content.len() as u64,
        content_hash,
        last_modified,
        source_agent_id: Some(agent_id.to_string()),
    })
}

/// Build skill info for a directory-based skill
fn build_dir_skill_info(
    path: &Path,
    name: &str,
    agent_id: &str,
) -> Result<SkillInfo, String> {
    let skill_md = path.join("SKILL.md");
    let has_skill_md = skill_md.exists();

    let meta = if has_skill_md {
        parse_skill_meta(&skill_md)?
    } else {
        None
    };

    let mut file_count = 0u64;
    let mut total_size = 0u64;
    let mut hash_input = String::new();

    collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;

    let content_hash = calculate_hash(hash_input.as_bytes());

    let last_modified = get_modified_time(path);

    Ok(SkillInfo {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        is_file: false,
        has_skill_md,
        meta,
        file_count,
        size_bytes: total_size,
        content_hash,
        last_modified,
        source_agent_id: Some(agent_id.to_string()),
    })
}

/// Get modified time as ISO 8601 string
fn get_modified_time(path: &Path) -> Option<String> {
    path.metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            let secs_since_epoch = t
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs_since_epoch as i64, 0)?;
            Some(datetime.to_rfc3339())
        })
}

/// Parse skill metadata from SKILL.md file
fn parse_skill_meta(path: &Path) -> Result<Option<SkillMeta>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    Ok(parse_skill_meta_from_content(content.as_bytes()))
}

/// Parse skill metadata from content bytes
fn parse_skill_meta_from_content(content: &[u8]) -> Option<SkillMeta> {
    let text = std::str::from_utf8(content).ok()?;

    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let rest = &trimmed[3..];
    let end_idx = rest.find("---")?;
    let yaml_str = &rest[..end_idx];

    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let description = parsed
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version = parsed.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());

    let author = parsed.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());

    let tags = parsed
        .get("tags")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let dependencies = parsed.get("dependencies").and_then(|v| {
        if let Some(map) = v.as_mapping() {
            let mut result = HashMap::new();
            for (k, v) in map {
                if let (Some(key_str), Some(val_str)) = (k.as_str(), v.as_str()) {
                    result.insert(key_str.to_string(), val_str.to_string());
                }
            }
            Some(result)
        } else {
            None
        }
    });

    if name.is_empty() {
        return None;
    }

    Some(SkillMeta {
        name,
        description,
        version,
        author,
        tags,
        dependencies,
    })
}

/// Install a skill by copying from source to target
pub fn install_skill(source: &str, target_path: &str) -> Result<OperationResult, String> {
    let src = Path::new(source);
    let target = Path::new(target_path);

    if !src.exists() {
        return Err(format!("Source does not exist: {}", source));
    }

    if target.exists() {
        return Err(format!("Target already exists: {}", target_path));
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    if src.is_file() {
        std::fs::copy(src, target)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
    } else {
        copy_dir_recursive(src, target)?;
    }

    Ok(OperationResult {
        success: true,
        message: format!("Skill installed successfully to {}", target_path),
        affected_paths: vec![target_path.to_string()],
    })
}

/// Update a skill by deleting target and installing fresh
pub fn update_skill(source: &str, target_path: &str) -> Result<OperationResult, String> {
    let target = Path::new(target_path);

    // Remove existing if present
    if target.exists() {
        uninstall_skill(target_path)?;
    }

    install_skill(source, target_path)
}

/// Uninstall a skill by deleting it
pub fn uninstall_skill(skill_path: &str) -> Result<OperationResult, String> {
    let path = Path::new(skill_path);

    if !path.exists() {
        return Ok(OperationResult {
            success: true,
            message: "Skill was already removed".to_string(),
            affected_paths: vec![skill_path.to_string()],
        });
    }

    if path.is_file() {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file: {}", e))?;
    } else {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory: {}", e))?;
    }

    Ok(OperationResult {
        success: true,
        message: format!("Skill uninstalled successfully from {}", skill_path),
        affected_paths: vec![skill_path.to_string()],
    })
}

/// Batch operation on multiple skills
pub fn batch_operate(operations: Vec<SkillOperation>) -> Vec<OperationResult> {
    operations
        .into_iter()
        .map(|op| {
            match op.operation_type {
                OperationType::Install => install_skill(&op.source, &op.target_path),
                OperationType::Update => update_skill(&op.source, &op.target_path),
                OperationType::Uninstall => uninstall_skill(&op.target_path),
            }
            .unwrap_or_else(|e| OperationResult {
                success: false,
                message: e,
                affected_paths: vec![],
            })
        })
        .collect()
}

/// Verify skill integrity by comparing hashes
pub fn verify_skill_integrity(
    skill_path: &str,
    expected_hash: &str,
) -> Result<bool, String> {
    let path = Path::new(skill_path);

    if !path.exists() {
        return Err(format!("Skill path does not exist: {}", skill_path));
    }

    let actual_hash = if path.is_file() {
        let content = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        calculate_hash(&content)
    } else {
        let mut file_count = 0u64;
        let mut total_size = 0u64;
        let mut hash_input = String::new();
        collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;
        calculate_hash(hash_input.as_bytes())
    };

    Ok(actual_hash == expected_hash)
}

/// Get overview of skills across projects
pub fn get_projects_overview(
    project_paths: &[String],
    agent_ids: &[String],
    _global_paths: &HashMap<String, String>,
    project_patterns: &HashMap<String, String>,
) -> Result<Vec<ProjectOverview>, String> {
    let mut overviews = Vec::new();

    for project_path in project_paths {
        let project_name = Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(project_path)
            .to_string();

        let mut agent_skills_count = HashMap::new();

        for agent_id in agent_ids {
            let pattern = project_patterns
                .get(agent_id)
                .map(|p| p.as_str())
                .unwrap_or("{project}/.claude/skills");

            let skills_dir = pattern.replace("{project}", project_path);

            if let Ok(scan_result) = scan_skills(&skills_dir, agent_id, SkillScope::Project) {
                agent_skills_count.insert(agent_id.clone(), scan_result.total);
            }
        }

        overviews.push(ProjectOverview {
            project_path: project_path.clone(),
            project_name,
            agent_skills_count,
            local_count: 0,
            same_count: 0,
            outdated_count: 0,
            remote_only_count: 0,
            last_modified: None,
            readme_preview: None,
        });
    }

    Ok(overviews)
}
pub fn get_rich_projects_overview(
    project_paths: &[String],
    agent_id: &str,
    project_pattern: &str,
    repos: &[(String, String)], // (repo_id, cache_path)
) -> Result<Vec<ProjectOverview>, String> {
    // 1. Scan all remote repos once, build a unified map
    let mut remote_map: HashMap<String, (SkillInfo, String)> = HashMap::new();
    for (repo_id, cache_path) in repos {
        if let Ok(remote_result) = scan_remote_skills(cache_path, repo_id) {
            for skill in remote_result.skills {
                remote_map
                    .entry(skill.name.clone())
                    .or_insert_with(|| (skill, repo_id.clone()));
            }
        }
    }

    let mut overviews = Vec::new();

    for project_path in project_paths {
        let project_name = Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(project_path)
            .to_string();

        // 2. Resolve local skill dir for this project
        let skills_dir = format!("{}/{}/skills", project_path, project_pattern);

        let local_result = scan_skills(&skills_dir, agent_id, SkillScope::Project)
            .unwrap_or_else(|_| ScanResult {
                agent_id: agent_id.to_string(),
                scope: SkillScope::Project,
                skills: Vec::new(),
                total: 0,
            });

        let local_map: HashMap<String, SkillInfo> = local_result
            .skills
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();

        // 3. Count statuses
        let local_count = local_map.len() as u32;
        let mut same_count = 0u32;
        let mut outdated_count = 0u32;

        for (name, local_skill) in &local_map {
            if let Some((remote_skill, _)) = remote_map.get(name) {
                if local_skill.content_hash == remote_skill.content_hash {
                    same_count += 1;
                } else {
                    outdated_count += 1;
                }
            }
        }

        let remote_names_in_local: std::collections::HashSet<&str> =
            local_map.keys().map(|s| s.as_str()).collect();
        let mut remote_only_count = 0u32;
        for name in remote_map.keys() {
            if !remote_names_in_local.contains(name.as_str()) {
                remote_only_count += 1;
            }
        }

        // 4. Find most recent last_modified
        let last_modified = local_map
            .values()
            .filter_map(|s| s.last_modified.as_ref())
            .max()
            .cloned();

        // 5. Try reading README from project root
        let readme_preview = try_read_readme(project_path);

        let mut agent_skills_count = HashMap::new();
        agent_skills_count.insert(agent_id.to_string(), local_count as u64);

        overviews.push(ProjectOverview {
            project_path: project_path.clone(),
            project_name,
            agent_skills_count,
            local_count,
            same_count,
            outdated_count,
            remote_only_count,
            last_modified,
            readme_preview,
        });
    }

    Ok(overviews)
}

/// Try reading README from project root, return first 5 lines.
fn try_read_readme(project_path: &str) -> Option<String> {
    let readme_names = ["README.md", "readme.md", "Readme.md", "README", "readme"];
    let base = Path::new(project_path);

    for name in readme_names {
        let path = base.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let preview: String = content
                    .lines()
                    .take(5)
                    .collect::<Vec<&str>>()
                    .join("\n");
                if !preview.trim().is_empty() {
                    return Some(preview);
                }
            }
        }
    }
    None
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;
    }

    let entries = std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_file() {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {} to {}: {}", src_path.display(), dst_path.display(), e))?;
        } else if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Build skill comparisons by pairing local skills with remote skills.
pub fn build_skill_comparisons(
    local_dir: &str,
    repos: &[(String, String)], // (repo_id, cache_path)
) -> Result<Vec<SkillComparison>, String> {
    // 1. Scan local skills
    let local_result = scan_skills(local_dir, "", SkillScope::Global)?;
    let local_map: HashMap<String, SkillInfo> = local_result
        .skills
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    // 2. Scan remote skills from all repos, build map of name -> (SkillInfo, repo_id)
    let mut remote_map: HashMap<String, (SkillInfo, String)> = HashMap::new();
    for (repo_id, cache_path) in repos {
        if let Ok(remote_result) = scan_remote_skills(cache_path, repo_id) {
            for skill in remote_result.skills {
                remote_map.entry(skill.name.clone()).or_insert_with(|| {
                    (skill, repo_id.clone())
                });
            }
        }
    }

    // 3. Build comparisons
    let mut comparisons = Vec::new();
    let mut all_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for name in local_map.keys() {
        all_names.insert(name.clone());
    }
    for name in remote_map.keys() {
        all_names.insert(name.clone());
    }

    for name in all_names {
        let local = local_map.get(&name).cloned();
        let remote_entry = remote_map.get(&name);
        let remote = remote_entry.map(|(s, _)| s.clone());
        let source_repo = remote_entry.map(|(_, repo_id)| repo_id.clone());

        let status = match (&local, &remote) {
            (Some(l), Some(r)) => {
                if l.content_hash == r.content_hash {
                    ComparisonStatus::Same
                } else {
                    ComparisonStatus::Outdated
                }
            }
            (Some(_), None) => ComparisonStatus::LocalOnly,
            (None, Some(_)) => ComparisonStatus::RemoteOnly,
            (None, None) => ComparisonStatus::LocalOnly, // Should not happen, but handle it
        };

        comparisons.push(SkillComparison {
            name,
            status,
            local,
            remote,
            source_repo,
        });
    }

    // Sort: Outdated first, then RemoteOnly, then Same, then LocalOnly
    comparisons.sort_by(|a, b| {
        let order = |s: &ComparisonStatus| match s {
            ComparisonStatus::Outdated => 0,
            ComparisonStatus::RemoteOnly => 1,
            ComparisonStatus::Same => 2,
            ComparisonStatus::LocalOnly => 3,
        };
        order(&a.status).cmp(&order(&b.status))
    });

    Ok(comparisons)
}

/// Build file-level diff between local and remote skill directories.
pub fn build_skill_diff(
    local_path: &str,
    remote_path: &str,
) -> Result<SkillDiffResult, String> {
    let local = Path::new(local_path);
    let remote = Path::new(remote_path);

    let mut local_files: HashMap<String, (String, u64)> = HashMap::new();
    let mut remote_files: HashMap<String, (String, u64)> = HashMap::new();

    if local.exists() {
        collect_file_hashes(local, local, &mut local_files)?;
    }
    if remote.exists() {
        collect_file_hashes(remote, remote, &mut remote_files)?;
    }

    let mut all_paths: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for p in local_files.keys() {
        all_paths.insert(p.clone());
    }
    for p in remote_files.keys() {
        all_paths.insert(p.clone());
    }

    let mut file_diffs = Vec::new();
    let mut added = 0u32;
    let mut removed = 0u32;
    let mut modified = 0u32;
    let mut unchanged = 0u32;

    for rel_path in all_paths {
        let local_entry = local_files.get(&rel_path);
        let remote_entry = remote_files.get(&rel_path);

        let (diff_type, local_size, remote_size) = match (local_entry, remote_entry) {
            (Some((lh, ls)), Some((rh, rs))) => {
                if lh == rh {
                    unchanged += 1;
                    (DiffStatus::Same, Some(*ls), Some(*rs))
                } else {
                    modified += 1;
                    (DiffStatus::Modified, Some(*ls), Some(*rs))
                }
            }
            (Some((_, ls)), None) => {
                removed += 1;
                (DiffStatus::Removed, Some(*ls), None)
            }
            (None, Some((_, rs))) => {
                added += 1;
                (DiffStatus::Added, None, Some(*rs))
            }
            (None, None) => {
                // Should not happen since all_paths is built from both maps
                (DiffStatus::Same, None, None)
            }
        };

        let file_name = Path::new(&rel_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&rel_path)
            .to_string();

        file_diffs.push(FileDiffEntry {
            path: rel_path,
            file_name,
            diff_type,
            local_size,
            remote_size,
        });
    }

    Ok(SkillDiffResult {
        skill_name: local
            .file_name()
            .and_then(|n| n.to_str())
            .or_else(|| remote.file_name().and_then(|n| n.to_str()))
            .unwrap_or("unknown")
            .to_string(),
        file_diffs,
        added_count: added,
        removed_count: removed,
        modified_count: modified,
        unchanged_count: unchanged,
    })
}

/// Collect relative file paths with their hashes into a map.
fn collect_file_hashes(
    base: &Path,
    current: &Path,
    result: &mut HashMap<String, (String, u64)>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(current)
        .map_err(|e| format!("Failed to read dir {}: {}", current.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_file() {
            let rel = path.strip_prefix(base)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            let rel = rel.replace('\\', "/");

            let content = std::fs::read(&path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let size = content.len() as u64;
            let hash = calculate_hash(&content);
            result.insert(rel, (hash, size));
        } else if path.is_dir() {
            collect_file_hashes(base, &path, result)?;
        }
    }

    Ok(())
}

/// Get content of a file from both local and remote skill directories for line diff.
pub fn get_diff_file_content(
    local_skill_path: &str,
    remote_skill_path: &str,
    rel_file_path: &str,
) -> Result<DiffFileContent, String> {
    let normalized = rel_file_path.replace('\\', "/");

    let local_content = {
        let path = Path::new(local_skill_path).join(&normalized);
        if path.exists() {
            Some(std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read local file: {}", e))?)
        } else {
            None
        }
    };

    let remote_content = {
        let path = Path::new(remote_skill_path).join(&normalized);
        if path.exists() {
            Some(std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read remote file: {}", e))?)
        } else {
            None
        }
    };

    Ok(DiffFileContent {
        local_content,
        remote_content,
    })
}

// --- Skillbase dependency management ---

/// Parse skillbase.json from a project directory
pub fn parse_skillbase_manifest(project_path: &str) -> Result<SkillbaseManifest, String> {
    let path = Path::new(project_path).join("skillbase.json");
    if !path.exists() {
        return Err("skillbase.json not found in project directory".to_string());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read skillbase.json: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse skillbase.json: {}", e))
}

/// Simple semver range check: "^1.2.3" matches same major, ">=1.0.0" is >=, "*" matches all
fn semver_matches(version: &Option<String>, range: &str) -> bool {
    let version_str = match version {
        Some(v) if !v.is_empty() => v.as_str(),
        _ => return range == "*",
    };

    if range == "*" {
        return true;
    }

    let parse_ver = |s: &str| -> Vec<u32> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect::<Vec<_>>()
    };

    let ver_parts = parse_ver(version_str);

    if range.starts_with('^') {
        let range_parts = parse_ver(&range[1..]);
        if range_parts.is_empty() {
            return true;
        }
        // ^major.minor.patch → same major, >= range
        if ver_parts.is_empty() {
            return false;
        }
        if ver_parts[0] != range_parts[0] {
            return false;
        }
        // Must be >= range within the same major
        for i in 1..range_parts.len().min(ver_parts.len()) {
            if ver_parts[i] < range_parts[i] {
                return false;
            }
            if ver_parts[i] > range_parts[i] {
                return true;
            }
        }
        true
    } else if range.starts_with('~') {
        let range_parts = parse_ver(&range[1..]);
        if range_parts.is_empty() {
            return true;
        }
        if ver_parts.is_empty() {
            return false;
        }
        // ~major.minor.patch → same major.minor, >= range
        if ver_parts.len() < 2 || range_parts.len() < 2 {
            return ver_parts.get(0) == range_parts.get(0);
        }
        if ver_parts[0] != range_parts[0] || ver_parts[1] != range_parts[1] {
            return false;
        }
        if ver_parts.len() >= 3 && range_parts.len() >= 3 && ver_parts[2] < range_parts[2] {
            return false;
        }
        true
    } else if range.starts_with(">=") {
        let range_parts = parse_ver(&range[2..]);
        if range_parts.is_empty() {
            return true;
        }
        for i in 0..range_parts.len().min(ver_parts.len()) {
            if ver_parts[i] > range_parts[i] {
                return true;
            }
            if ver_parts[i] < range_parts[i] {
                return false;
            }
        }
        ver_parts.len() >= range_parts.len()
    } else {
        // Exact match
        version_str == range
    }
}

/// Parse a skill reference like "@author/name" or just "name" into (author, name)
fn parse_skill_reference(reference: &str) -> (String, String) {
    if let Some(stripped) = reference.strip_prefix('@') {
        if let Some(slash_pos) = stripped.find('/') {
            let author = &stripped[..slash_pos];
            let name = &stripped[slash_pos + 1..];
            return (author.to_string(), name.to_string());
        }
    }
    (String::new(), reference.to_string())
}

/// Resolve all dependencies declared in a project's skillbase.json.
/// `skill_dir` is the local skill directory (e.g. `{project}/.claude/skills`).
/// `repos` is a list of (repo_id, cache_path) pairs.
pub fn resolve_skillbase(
    skill_dir: &str,
    repos: &[(String, String)],
) -> Result<SkillbaseResolution, String> {
    let manifest = parse_skillbase_manifest(
        Path::new(skill_dir)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
            .as_str(),
    )?;

    // Filter repos by manifest registry if specified
    let filtered_repos: Vec<&(String, String)> = if let Some(ref registry_url) = manifest.registry
    {
        let matched: Vec<&(String, String)> = repos
            .iter()
            .filter(|(_, cache_path)| {
                // Can't directly match URL from cache_path; use all repos
                let _ = registry_url;
                true
            })
            .collect();
        if matched.is_empty() {
            repos.iter().collect()
        } else {
            matched
        }
    } else {
        repos.iter().collect()
    };

    // Build index of remote skills: name → (version, repo_id)
    let mut remote_index: HashMap<String, (Option<String>, String)> = HashMap::new();
    for (repo_id, cache_path) in &filtered_repos {
        if let Ok(remote_result) = scan_remote_skills(cache_path, repo_id) {
            for skill in remote_result.skills {
                let version = skill.meta.as_ref().and_then(|m| m.version.clone());
                remote_index
                    .entry(skill.name.clone())
                    .or_insert_with(|| (version, repo_id.clone()));
            }
        }
    }

    // Build index of local skills: name → version
    let local_result = scan_skills(skill_dir, "", SkillScope::Global).unwrap_or_else(|_| ScanResult {
        agent_id: String::new(),
        scope: SkillScope::Global,
        skills: Vec::new(),
        total: 0,
    });
    let local_map: HashMap<String, Option<String>> = local_result
        .skills
        .iter()
        .map(|s| (s.name.clone(), s.meta.as_ref().and_then(|m| m.version.clone())))
        .collect();

    let mut dependencies = Vec::new();
    let mut satisfied_count = 0;
    let mut missing_count = 0;
    let mut mismatch_count = 0;
    let mut outdated_count = 0;

    for (reference, version_range) in &manifest.skills {
        let (_author, skill_name) = parse_skill_reference(reference);
        let installed_version = local_map.get(&skill_name).cloned().flatten();
        let resolved_version = remote_index.get(&skill_name).map(|(v, _)| v.clone()).flatten();

        let status = match (&installed_version, &resolved_version) {
            (Some(_), _) if !semver_matches(&installed_version, version_range) => {
                DependencyStatus::VersionMismatch
            }
            (Some(inst_v), Some(res_v))
                if res_v != inst_v && semver_matches(&Some(res_v.clone()), version_range) =>
            {
                DependencyStatus::Outdated
            }
            (Some(_), _) => DependencyStatus::Satisfied,
            (None, _) => DependencyStatus::Missing,
        };

        match &status {
            DependencyStatus::Satisfied => satisfied_count += 1,
            DependencyStatus::Missing => missing_count += 1,
            DependencyStatus::VersionMismatch => mismatch_count += 1,
            DependencyStatus::Outdated => outdated_count += 1,
        }

        dependencies.push(DependencyEntry {
            reference: reference.clone(),
            skill_name,
            version_range: version_range.clone(),
            resolved_version,
            installed_version,
            status,
        });
    }

    let total_count = dependencies.len();
    Ok(SkillbaseResolution {
        manifest,
        dependencies,
        total_count,
        satisfied_count,
        missing_count,
        mismatch_count,
        outdated_count,
    })
}

/// Sync skillbase dependencies: install missing/mismatched/outdated skills from repos.
/// `skill_dir` is the local skill directory. `repos` is (repo_id, cache_path) pairs.
pub fn sync_skillbase(
    skill_dir: &str,
    repos: &[(String, String)],
) -> Result<Vec<SkillbaseSyncResult>, String> {
    let resolution = resolve_skillbase(skill_dir, repos)?;

    // Build remote skill lookup: name → (full_path, repo_id)
    let mut remote_lookup: HashMap<String, (String, String)> = HashMap::new();
    for (repo_id, cache_path) in repos {
        if let Ok(remote_result) = scan_remote_skills(cache_path, repo_id) {
            for skill in remote_result.skills {
                remote_lookup
                    .entry(skill.name.clone())
                    .or_insert_with(|| (skill.path.clone(), repo_id.clone()));
            }
        }
    }

    let mut results = Vec::new();
    for dep in &resolution.dependencies {
        match dep.status {
            DependencyStatus::Satisfied => {
                results.push(SkillbaseSyncResult {
                    reference: dep.reference.clone(),
                    success: true,
                    error: None,
                });
            }
            DependencyStatus::Missing | DependencyStatus::VersionMismatch | DependencyStatus::Outdated => {
                if let Some((remote_path, _repo_id)) = remote_lookup.get(&dep.skill_name) {
                    let target_path = format!("{}/{}", skill_dir, dep.skill_name);

                    // Remove existing if present
                    let target = Path::new(&target_path);
                    if target.exists() {
                        let _ = uninstall_skill(&target_path);
                    }

                    match install_skill(remote_path, &target_path) {
                        Ok(_) => results.push(SkillbaseSyncResult {
                            reference: dep.reference.clone(),
                            success: true,
                            error: None,
                        }),
                        Err(e) => results.push(SkillbaseSyncResult {
                            reference: dep.reference.clone(),
                            success: false,
                            error: Some(e),
                        }),
                    }
                } else {
                    results.push(SkillbaseSyncResult {
                        reference: dep.reference.clone(),
                        success: false,
                        error: Some("Not found in any repo".to_string()),
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Generate skillbase.json content from currently installed skills.
pub fn generate_skillbase(
    skill_dir: &str,
    repos: &[(String, String)],
) -> Result<String, String> {
    let local_result = scan_skills(skill_dir, "", SkillScope::Global).unwrap_or_else(|_| ScanResult {
        agent_id: String::new(),
        scope: SkillScope::Global,
        skills: Vec::new(),
        total: 0,
    });

    let mut skills = HashMap::new();
    for skill in &local_result.skills {
        let author = skill
            .meta
            .as_ref()
            .and_then(|m| m.author.as_deref())
            .unwrap_or("local");
        let version = skill
            .meta
            .as_ref()
            .and_then(|m| m.version.as_deref())
            .unwrap_or("*");
        let version_range = if version == "*" {
            "*".to_string()
        } else {
            format!("^{}", version)
        };
        skills.insert(format!("@{}/{}", author, skill.name), version_range);
    }

    // Pick registry URL from the repo with the most matching skills
    let registry_url = if !local_result.skills.is_empty() {
        let local_names: std::collections::HashSet<&str> =
            local_result.skills.iter().map(|s| s.name.as_str()).collect();

        repos
            .iter()
            .filter_map(|(repo_id, cache_path)| {
                if let Ok(remote) = scan_remote_skills(cache_path, repo_id) {
                    let remote_names: std::collections::HashSet<&str> =
                        remote.skills.iter().map(|s| s.name.as_str()).collect();
                    let match_count = local_names.intersection(&remote_names).count();
                    if match_count > 0 {
                        Some(match_count)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .max()
            .and_then(|_| {
                // Return first repo URL as registry hint (we don't have URL in repos tuple)
                None
            })
    } else {
        None
    };

    let project_name = Path::new(skill_dir)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let manifest = SkillbaseManifest {
        schema_version: 1,
        name: project_name,
        version: "1.0.0".to_string(),
        skills,
        registry: registry_url,
    };

    serde_json::to_string_pretty(&manifest).map_err(|e| format!("Serialize: {}", e))
}

/// Write skillbase.json content to the project directory
pub fn write_skillbase(project_path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(project_path).join("skillbase.json");
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write skillbase.json: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let content = b"hello world";
        let hash = calculate_hash(content);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_parse_skill_meta_from_content() {
        let content = b"---
name: Test Skill
description: A test skill
version: 1.0.0
author: Test Author
tags:
  - test
  - example
---
Some content here";

        let meta = parse_skill_meta_from_content(content).unwrap();
        assert_eq!(meta.name, "Test Skill");
        assert_eq!(meta.description, "A test skill");
        assert_eq!(meta.version, Some("1.0.0".to_string()));
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.tags, vec!["test", "example"]);
    }

    #[test]
    fn test_semver_matches() {
        assert!(semver_matches(&Some("1.0.0".to_string()), "*"));
        assert!(semver_matches(&Some("1.2.3".to_string()), "^1.0.0"));
        assert!(!semver_matches(&Some("2.0.0".to_string()), "^1.0.0"));
        assert!(semver_matches(&Some("1.2.3".to_string()), "~1.2.0"));
        assert!(!semver_matches(&Some("1.3.0".to_string()), "~1.2.0"));
        assert!(semver_matches(&Some("2.0.0".to_string()), ">=1.0.0"));
        assert!(semver_matches(&Some("1.0.0".to_string()), "1.0.0"));
        assert!(semver_matches(&None, "*"));
    }

    #[test]
    fn test_parse_skill_reference() {
        assert_eq!(
            parse_skill_reference("@author/my-skill"),
            ("author".to_string(), "my-skill".to_string())
        );
        assert_eq!(
            parse_skill_reference("my-skill"),
            (String::new(), "my-skill".to_string())
        );
    }
}
