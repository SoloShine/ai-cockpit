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
        });
    }

    Ok(overviews)
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
}
