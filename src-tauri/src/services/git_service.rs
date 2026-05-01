// src-tauri/src/services/git_service.rs
use std::path::Path;
use std::process::{Command, Stdio};

/// Check if a directory is a valid git repository
pub fn validate_repo(cache_path: &str) -> bool {
    let path = Path::new(cache_path);
    if !path.join(".git").exists() {
        return false;
    }
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Perform a fresh git clone with --depth 1
fn git_clone(remote_url: &str, cache: &Path) -> Result<(), String> {
    if cache.exists() {
        std::fs::remove_dir_all(cache)
            .map_err(|e| format!("Failed to clean cache dir: {}", e))?;
    }
    let parent = cache.parent().ok_or("Invalid cache path")?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create parent dir: {}", e))?;

    let dir_name = cache
        .file_name()
        .ok_or("Invalid cache path")?
        .to_string_lossy()
        .to_string();

    let output = Command::new("git")
        .args(["clone", "--depth", "1", remote_url, &dir_name])
        .current_dir(parent)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(cache);
        return Err(format!("git clone failed: {}", stderr));
    }
    Ok(())
}

/// Sync a repository: clone if missing, pull if exists, with fallback strategies.
///
/// 1. Valid repo → `git pull --ff-only`
/// 2. Pull fails → `git fetch origin` + `git reset --hard origin/HEAD`
/// 3. All fail → delete and `git clone --depth 1`
pub fn sync_repo(url: &str, cache_path: &str) -> Result<(), String> {
    let cache = Path::new(cache_path);

    if validate_repo(cache_path) {
        let pull_output = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(cache)
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run git pull: {}", e))?;

        if pull_output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&pull_output.stderr);

        let fetch_output = Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(cache)
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run git fetch: {}", e))?;

        if !fetch_output.status.success() {
            eprintln!(
                "Pull/fetch failed for '{}', re-cloning: {}",
                cache_path, stderr
            );
            return git_clone(url, cache);
        }

        let reset_output = Command::new("git")
            .args(["reset", "--hard", "origin/HEAD"])
            .current_dir(cache)
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run git reset: {}", e))?;

        if !reset_output.status.success() {
            let rs = String::from_utf8_lossy(&reset_output.stderr);
            return Err(format!("git reset failed: {}", rs));
        }
    } else {
        git_clone(url, cache)?;
    }
    Ok(())
}

/// Get the ISO 8601 timestamp of the latest commit
pub fn get_latest_commit_time(cache_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%aI"])
        .current_dir(cache_path)
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get commit time".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_repo_nonexistent() {
        assert!(!validate_repo("/nonexistent/path/that/does/not/exist"));
    }

    #[test]
    fn test_get_latest_commit_time_nonexistent() {
        assert!(get_latest_commit_time("/nonexistent/path").is_err());
    }
}
