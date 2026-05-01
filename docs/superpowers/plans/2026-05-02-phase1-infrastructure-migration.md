# Phase 1: Infrastructure Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Git sync, remote skill discovery, and config import/export to the ai-cockpit skills plugin.

**Architecture:** Backend-first approach — Rust models → services → commands → frontend types → store → UI. The existing settings UI (SettingsView with tabs) is already functional; we extend it with new backend capabilities and enhance the existing RepoPanel component.

**Tech Stack:** Rust (std::process::Command for git CLI, serde, tokio), TypeScript, Vue 3, Naive UI, Pinia

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src-tauri/src/models/git_sync.rs` | SyncResult, RemoteSkillInfo, RemoteSkillDetail structs |
| `src-tauri/src/services/git_service.rs` | Git clone/pull/validate/fallback operations |
| `src-tauri/src/commands/git_sync.rs` | IPC command handlers for git sync and remote skills |

### Modified Files
| File | Change |
|------|--------|
| `src-tauri/src/models/mod.rs` | Add `pub mod git_sync;` |
| `src-tauri/src/services/mod.rs` | Add `pub mod git_service;` |
| `src-tauri/src/commands/mod.rs` | Add `pub mod git_sync;` |
| `src-tauri/src/commands/settings.rs` | Add export_config, import_config commands |
| `src-tauri/src/lib.rs` | Register new commands in invoke_handler |
| `src/plugins/skills/types.ts` | Add SyncResult, RemoteSkillInfo, RemoteSkillDetail interfaces |
| `src/plugins/settings/store.ts` | Add syncAllRepos, getRemoteSkills, exportConfig, importConfig methods |
| `src/plugins/skills/components/RepoPanel.vue` | Add sync button, status display, last sync time |
| `src/plugins/skills/i18n/zh-CN.json` | Add sync-related translation keys |
| `src/plugins/skills/i18n/en-US.json` | Add sync-related translation keys |

---

## Task 1: Git Sync Models (Rust)

**Files:**
- Create: `src-tauri/src/models/git_sync.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: Create the git_sync models file**

```rust
// src-tauri/src/models/git_sync.rs
use serde::{Deserialize, Serialize};

use super::skills::FileEntry;

/// Result of syncing a single repository
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub repo_id: String,
    pub success: bool,
    pub message: String,
    pub skill_count: u32,
}

/// Summary of a skill available in a remote repository
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkillInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub source_repo: String,
    pub skill_type: String,
}

/// Detailed info about a remote skill including file tree and hash
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkillDetail {
    pub info: RemoteSkillInfo,
    pub files: Vec<FileEntry>,
    pub content_hash: String,
}
```

- [ ] **Step 2: Register the module in models/mod.rs**

Add `pub mod git_sync;` to `src-tauri/src/models/mod.rs`:

```rust
pub mod git_sync;
pub mod skills;
```

- [ ] **Step 3: Verify compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9\src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models/git_sync.rs src-tauri/src/models/mod.rs
git commit -m "feat(models): add git sync types — SyncResult, RemoteSkillInfo, RemoteSkillDetail"
```

---

## Task 2: Git Service (Rust)

**Files:**
- Create: `src-tauri/src/services/git_service.rs`
- Modify: `src-tauri/src/services/mod.rs`

Adapted from field-skill-manage's `services/git_service.rs`. Uses `std::process::Command` to invoke system git CLI.

- [ ] **Step 1: Create the git_service file**

```rust
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
```

- [ ] **Step 2: Register the module in services/mod.rs**

Add `pub mod git_service;` to `src-tauri/src/services/mod.rs`:

```rust
pub mod git_service;
pub mod settings_service;
pub mod skills_service;
```

- [ ] **Step 3: Verify compilation and tests**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9\src-tauri && cargo test --lib services::git_service 2>&1 | tail -10`
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/services/git_service.rs src-tauri/src/services/mod.rs
git commit -m "feat(services): add git_service — clone/pull with 3-tier fallback"
```

---

## Task 3: Git Sync Commands (Rust)

**Files:**
- Create: `src-tauri/src/commands/git_sync.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

These IPC commands use `git_service` for git operations and `skills_service` for scanning remote skill directories.

- [ ] **Step 1: Create the git_sync commands file**

```rust
// src-tauri/src/commands/git_sync.rs
use crate::models::git_sync::*;
use crate::services::git_service;
use crate::services::settings_service::RepoConfig;
use crate::services::skills_service;

/// Expand a path: ~ → home dir, relative → prepend home dir, absolute → as-is
fn expand_path(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }
    if path.starts_with('~') {
        return path.replacen(
            "~",
            &dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string()),
            1,
        );
    }
    if path.starts_with('/') || (path.len() >= 2 && path.as_bytes()[1] == b':') {
        return path.to_string();
    }
    // Relative path → resolve against app data dir
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    format!("{}/.ai-cockpit/{}", home, path)
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
            let expanded = expand_path(&repo.cache_path);
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

    handles
        .into_iter()
        .filter_map(|handle| handle.ok())
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
/// Parses skills.json if available, falls back to scanning for SKILL.md.
#[tauri::command]
pub async fn get_remote_skills(
    repo_id: String,
    cache_path: String,
) -> Result<Vec<RemoteSkillInfo>, String> {
    let cache_path = expand_path(&cache_path);
    let path = std::path::Path::new(&cache_path);
    if !path.exists() {
        return Ok(vec![]);
    }

    let scan_result = skills_service::scan_skills(&cache_path, &repo_id, crate::models::skills::SkillScope::Global)?;

    Ok(scan_result
        .skills
        .into_iter()
        .map(|s| RemoteSkillInfo {
            name: s.name,
            description: s.meta.map(|m| m.description).unwrap_or_default(),
            version: s.meta.and_then(|m| m.version),
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
    let cache_path = expand_path(&cache_path);
    let skill_path = std::path::Path::new(&cache_path)
        .join(&skill_name);
    if !skill_path.exists() {
        return Err(format!("Skill '{}' not found in repo '{}'", skill_name, repo_id));
    }

    let files = skills_service::build_file_tree(skill_path.to_string_lossy().to_string().as_str())?;

    let scan_result = skills_service::scan_skills(&cache_path, &repo_id, crate::models::skills::SkillScope::Global)?;
    let skill_info = scan_result
        .skills
        .into_iter()
        .find(|s| s.name == skill_name)
        .ok_or_else(|| format!("Skill '{}' not found", skill_name))?;

    Ok(RemoteSkillDetail {
        info: RemoteSkillInfo {
            name: skill_info.name,
            description: skill_info.meta.map(|m| m.description).unwrap_or_default(),
            version: skill_info.meta.and_then(|m| m.version),
            source_repo: repo_id,
            skill_type: if skill_info.is_file { "file".to_string() } else { "directory".to_string() },
        },
        files,
        content_hash: skill_info.content_hash,
    })
}

/// Count skill directories/files in a cached repo
fn count_skills_in_dir(cache_path: &str) -> Result<u32, String> {
    let result = skills_service::scan_skills(cache_path, "", crate::models::skills::SkillScope::Global)?;
    Ok(result.total as u32)
}
```

- [ ] **Step 2: Register the module in commands/mod.rs**

Add `pub mod git_sync;` to `src-tauri/src/commands/mod.rs`:

```rust
pub mod core;
pub mod git_sync;
pub mod settings;
pub mod skills;
```

- [ ] **Step 3: Register commands in lib.rs invoke_handler**

Add these 3 commands to the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // Core
    commands::core::get_app_version,
    // Settings
    commands::settings::load_settings,
    commands::settings::save_settings,
    commands::settings::get_data_dir,
    commands::settings::open_in_explorer,
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
])
```

- [ ] **Step 4: Verify compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9\src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/git_sync.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add git_sync commands — sync_all_repos, get_remote_skills, get_remote_skill_detail"
```

---

## Task 4: Config Import/Export Commands (Rust)

**Files:**
- Modify: `src-tauri/src/commands/settings.rs`

The existing settings commands use `load_settings`/`save_settings` which take `AppHandle`. The import/export commands add portable path handling (home dir ↔ `${HOME}`).

- [ ] **Step 1: Add import/export commands to settings.rs**

Append these functions to `src-tauri/src/commands/settings.rs`:

```rust
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
```

- [ ] **Step 2: Register commands in lib.rs invoke_handler**

Add `export_config` and `import_config` to the Settings section in `src-tauri/src/lib.rs`:

```rust
// Settings
commands::settings::load_settings,
commands::settings::save_settings,
commands::settings::get_data_dir,
commands::settings::open_in_explorer,
commands::settings::export_config,
commands::settings::import_config,
```

- [ ] **Step 3: Verify compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9\src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/settings.rs src-tauri/src/lib.rs
git commit -m "feat(settings): add export_config and import_config commands with portable paths"
```

---

## Task 5: Frontend Types for Git Sync

**Files:**
- Modify: `src/plugins/skills/types.ts`

- [ ] **Step 1: Add SyncResult, RemoteSkillInfo, RemoteSkillDetail interfaces**

Append to `src/plugins/skills/types.ts`:

```typescript
/** Result of syncing a single repository */
export interface SyncResult {
  repoId: string
  success: boolean
  message: string
  skillCount: number
}

/** Summary of a skill in a remote repository */
export interface RemoteSkillInfo {
  name: string
  description: string
  version?: string
  sourceRepo: string
  skillType: 'file' | 'directory'
}

/** Detailed info about a remote skill */
export interface RemoteSkillDetail {
  info: RemoteSkillInfo
  files: FileEntry[]
  contentHash: string
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors related to the new types.

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/types.ts
git commit -m "feat(types): add SyncResult, RemoteSkillInfo, RemoteSkillDetail"
```

---

## Task 6: Settings Store Enhancements

**Files:**
- Modify: `src/plugins/settings/store.ts`

Add sync, import/export methods to the existing store. These methods call the new Rust backend commands.

- [ ] **Step 1: Add import for invoke and new types**

Add the `SyncResult` and `RemoteSkillInfo` imports at the top of `src/plugins/settings/store.ts`:

```typescript
import type { AppSettings, AppearanceSettings, AgentConfig, PluginSettings, RepoConfig } from "./types";
```

No new imports needed — `invoke` is already imported.

- [ ] **Step 2: Add sync and import/export methods**

Add these methods inside the `defineStore` callback, before the `return` statement, after the `updateRepo` function:

```typescript
  // --- Git Sync ---

  const syncResults = ref<import('@/plugins/skills/types').SyncResult[]>([]);
  const syncing = ref(false);

  async function syncAllRepos(): Promise<import('@/plugins/skills/types').SyncResult[]> {
    syncing.value = true;
    try {
      const results = await invoke<import('@/plugins/skills/types').SyncResult[]>("sync_all_repos", {
        repos: repos.value,
      });
      syncResults.value = results;

      // Update cache paths for repos that don't have one yet
      for (const result of results) {
        if (result.success) {
          const repo = repos.value.find((r) => r.id === result.repo_id);
          if (repo && !repo.cachePath) {
            updateRepo(repo.id, {
              cachePath: deriveCachePath(repo.id),
            });
          }
        }
      }
      return results;
    } finally {
      syncing.value = false;
    }
  }

  function deriveCachePath(repoId: string): string {
    // Cache in app data dir under repos/<repoId>
    return `repos/${repoId}`;
  }

  async function getRemoteSkills(repoId: string): Promise<import('@/plugins/skills/types').RemoteSkillInfo[]> {
    const repo = repos.value.find((r) => r.id === repoId);
    if (!repo) return [];
    return invoke<import('@/plugins/skills/types').RemoteSkillInfo[]>("get_remote_skills", {
      repoId,
      cachePath: repo.cachePath,
    });
  }

  // --- Config Portability ---

  async function exportConfig(): Promise<string> {
    const settings: AppSettings = {
      appearance: appearance.value,
      agents: agents.value,
      repos: repos.value,
      plugins: plugins.value,
      _meta: { version: 1, updatedAt: new Date().toISOString() },
    };
    return invoke<string>("export_config", { settings });
  }

  async function importConfig(json: string): Promise<void> {
    const settings = await invoke<AppSettings>("import_config", { json });
    appearance.value = settings.appearance;
    agents.value = settings.agents;
    repos.value = settings.repos ?? [];
    plugins.value = settings.plugins;
  }
```

- [ ] **Step 3: Expose new state and methods in the return object**

Add to the `return` object in the store:

```typescript
  return {
    // ... existing ...
    // Git Sync
    syncResults,
    syncing,
    syncAllRepos,
    getRemoteSkills,
    // Config Portability
    exportConfig,
    importConfig,
  };
```

- [ ] **Step 4: Verify TypeScript compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src/plugins/settings/store.ts
git commit -m "feat(settings): add syncAllRepos, getRemoteSkills, exportConfig, importConfig to store"
```

---

## Task 7: i18n Additions for Sync

**Files:**
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`

- [ ] **Step 1: Add sync-related keys to zh-CN.json**

Add a `"sync"` section inside the `"skills"` object in `src/plugins/skills/i18n/zh-CN.json`:

```json
    "sync": {
      "syncAll": "同步所有仓库",
      "syncing": "正在同步...",
      "syncSuccess": "同步完成：{success} 成功，{fail} 失败",
      "syncFail": "同步失败",
      "lastSync": "上次同步：{time}",
      "skillCount": "{count} 个 Skills",
      "noReposToSync": "没有已启用的仓库可同步"
    }
```

- [ ] **Step 2: Add sync-related keys to en-US.json**

Add a `"sync"` section inside the `"skills"` object in `src/plugins/skills/i18n/en-US.json`:

```json
    "sync": {
      "syncAll": "Sync All Repos",
      "syncing": "Syncing...",
      "syncSuccess": "Sync complete: {success} succeeded, {fail} failed",
      "syncFail": "Sync failed",
      "lastSync": "Last synced: {time}",
      "skillCount": "{count} Skills",
      "noReposToSync": "No enabled repos to sync"
    }
```

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/i18n/zh-CN.json src/plugins/skills/i18n/en-US.json
git commit -m "feat(i18n): add sync-related translation keys for zh-CN and en-US"
```

---

## Task 8: RepoPanel Enhancement — Sync UI

**Files:**
- Modify: `src/plugins/skills/components/RepoPanel.vue`

Add a "Sync All" button, per-repo sync status, and last sync time display.

- [ ] **Step 1: Replace the full RepoPanel.vue with enhanced version**

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import {
  NButton,
  NCard,
  NSpace,
  NText,
  NInput,
  NSwitch,
  NTag,
  NEmpty,
  useMessage,
  NPopconfirm,
  NSpin,
} from "naive-ui";
import { AddOutline, SyncOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/plugins/settings/store";
import type { SyncResult } from "@/plugins/skills/types";

const { t } = useI18n();
const store = useSettingsStore();
const message = useMessage();

const showAddForm = ref(false);
const newName = ref("");
const newUrl = ref("");

const syncing = computed(() => store.syncing);
const syncResults = computed(() => store.syncResults);

function getSyncResult(repoId: string): SyncResult | undefined {
  return syncResults.value.find((r) => r.repoId === repoId);
}

async function handleSyncAll() {
  if (store.repos.filter((r) => r.enabled).length === 0) {
    message.info(t("skills.sync.noReposToSync"));
    return;
  }
  try {
    const results = await store.syncAllRepos();
    const success = results.filter((r) => r.success).length;
    const fail = results.filter((r) => !r.success).length;
    if (fail > 0) {
      message.warning(t("skills.sync.syncSuccess", { success, fail }));
    } else {
      message.success(t("skills.sync.syncSuccess", { success, fail: 0 }));
    }
  } catch (e) {
    message.error(t("skills.sync.syncFail") + ": " + String(e));
  }
}

function handleAdd() {
  if (!newName.value.trim()) {
    message.warning(t("skills.repos.nameRequired"));
    return;
  }
  if (!newUrl.value.trim()) {
    message.warning(t("skills.repos.urlRequired"));
    return;
  }
  const id = `repo_${Date.now()}`;
  store.addRepo({
    id,
    name: newName.value.trim(),
    url: newUrl.value.trim(),
    cachePath: `repos/${id}`,
    enabled: true,
  });
  newName.value = "";
  newUrl.value = "";
  showAddForm.value = false;
  message.success(t("skills.repos.addSuccess"));
}

function handleDelete(id: string) {
  store.removeRepo(id);
  message.success(t("skills.repos.deleteSuccess"));
}
</script>

<template>
  <div>
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText strong style="font-size: 16px">{{ t("skills.repos.title") }}</NText>
      <NSpace>
        <NButton
          type="primary"
          ghost
          :loading="syncing"
          @click="handleSyncAll"
        >
          <template #icon><SyncOutline /></template>
          {{ syncing ? t("skills.sync.syncing") : t("skills.sync.syncAll") }}
        </NButton>
        <NButton @click="showAddForm = !showAddForm">
          <template #icon><AddOutline /></template>
          {{ t("skills.repos.addRepo") }}
        </NButton>
      </NSpace>
    </NSpace>

    <NCard v-if="showAddForm" size="small" style="margin-bottom: 16px">
      <NSpace vertical :size="12">
        <NInput
          v-model:value="newName"
          :placeholder="t('skills.repos.name')"
          size="small"
        />
        <NInput
          v-model:value="newUrl"
          :placeholder="t('skills.repos.url')"
          size="small"
        />
        <NSpace>
          <NButton size="small" type="primary" @click="handleAdd">
            {{ t("skills.repos.addRepo") }}
          </NButton>
          <NButton size="small" @click="showAddForm = false">
            {{ t("skills.repos.cancel") }}
          </NButton>
        </NSpace>
      </NSpace>
    </NCard>

    <NEmpty v-if="store.repos.length === 0" :description="t('skills.repos.noRepos')" />

    <NCard
      v-for="repo in store.repos"
      :key="repo.id"
      size="small"
      style="margin-bottom: 12px"
    >
      <template #header>
        <NSpace align="center">
          <span>{{ repo.name }}</span>
          <NTag :type="repo.enabled ? 'success' : 'default'" size="small">
            {{ repo.enabled ? t("skills.repos.enabled") : t("skills.repos.disabled") }}
          </NTag>
          <NTag
            v-if="getSyncResult(repo.id)"
            :type="getSyncResult(repo.id)!.success ? 'success' : 'error'"
            size="small"
          >
            {{ getSyncResult(repo.id)!.success
              ? t("skills.sync.skillCount", { count: getSyncResult(repo.id)!.skillCount })
              : t("skills.sync.syncFail")
            }}
          </NTag>
        </NSpace>
      </template>
      <NSpace vertical :size="8">
        <NText depth="3" style="font-size: 13px; word-break: break-all">
          {{ repo.url }}
        </NText>
        <NSpace align="center" justify="space-between">
          <NSwitch
            :value="repo.enabled"
            @update:value="store.updateRepo(repo.id, { enabled: $event })"
          >
            <template #checked>{{ t("skills.repos.enabled") }}</template>
          </NSwitch>
          <NPopconfirm @positive-click="handleDelete(repo.id)">
            <template #trigger>
              <NButton size="tiny" type="error" quaternary>
                {{ t("skills.repos.delete") }}
              </NButton>
            </template>
            {{ t("skills.repos.deleteConfirm") }}
          </NPopconfirm>
        </NSpace>
      </NSpace>
    </NCard>
  </div>
</template>
```

- [ ] **Step 2: Verify compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/components/RepoPanel.vue
git commit -m "feat(skills): enhance RepoPanel with sync button and status display"
```

---

## Task 9: Settings View — Import/Export Buttons

**Files:**
- Modify: `src/plugins/settings/panels/AboutPanel.vue`

Add import/export config buttons to the existing About panel.

- [ ] **Step 1: Add import/export buttons to AboutPanel.vue**

Replace the full content of `src/plugins/settings/panels/AboutPanel.vue` with:

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NDescriptions, NDescriptionsItem, NButton, NSpace, NText, NTag, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";

const { t } = useI18n();
const message = useMessage();
const store = useSettingsStore();

const version = ref("loading...");
const dataDir = ref("loading...");

onMounted(async () => {
  try {
    version.value = await invoke<string>("get_app_version");
    dataDir.value = await invoke<string>("get_data_dir");
  } catch (e) {
    version.value = "error: " + String(e);
    dataDir.value = "error";
  }
});

async function openDataDir() {
  try {
    await invoke("open_in_explorer", { path: dataDir.value });
  } catch (e) {
    message.error("Cannot open directory: " + String(e));
  }
}

async function openLogs() {
  await openDataDir();
}

function checkUpdates() {
  message.info("Check for updates will be available in a future version");
}

async function handleExport() {
  try {
    const json = await store.exportConfig();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "ai-cockpit-settings.json";
    a.click();
    URL.revokeObjectURL(url);
    message.success("Config exported");
  } catch (e) {
    message.error("Export failed: " + String(e));
  }
}

async function handleImport() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".json";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const json = await file.text();
      await store.importConfig(json);
      message.success("Config imported");
    } catch (e) {
      message.error("Import failed: " + String(e));
    }
  };
  input.click();
}
</script>

<template>
  <div>
    <NDescriptions label-placement="left" bordered :column="1" title="AI Cockpit">
      <NDescriptionsItem :label="t('settings.about.version')">
        <NTag type="success">v{{ version }}</NTag>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.dataDir')">
        <NSpace align="center">
          <NText code>{{ dataDir }}</NText>
          <NButton size="tiny" @click="openDataDir">
            {{ t("settings.about.openInExplorer") }}
          </NButton>
        </NSpace>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.techStack')">
        <NSpace>
          <NTag>Tauri 2</NTag>
          <NTag>Vue 3</NTag>
          <NTag>TypeScript</NTag>
          <NTag>Naive UI</NTag>
          <NTag>Rust</NTag>
        </NSpace>
      </NDescriptionsItem>
    </NDescriptions>

    <NSpace style="margin-top: 16px">
      <NButton @click="checkUpdates">{{ t("settings.about.checkUpdates") }}</NButton>
      <NButton @click="openLogs">{{ t("settings.about.openLogs") }}</NButton>
      <NButton type="primary" ghost @click="handleExport">Export Config</NButton>
      <NButton ghost @click="handleImport">Import Config</NButton>
    </NSpace>
  </div>
</template>
```

- [ ] **Step 2: Verify compilation**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/plugins/settings/panels/AboutPanel.vue
git commit -m "feat(settings): add export/import config buttons to About panel"
```

---

## Task 10: Full Build Verification

**Files:** None (verification only)

- [ ] **Step 1: Run Rust tests**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9\src-tauri && cargo test --lib 2>&1 | tail -15`
Expected: All tests pass.

- [ ] **Step 2: Run frontend type check**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors.

- [ ] **Step 3: Build the full app**

Run: `cd D:\Project\ai-cockpit\.claude\worktrees\funny-cohen-e2bcf9 && npm run build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 4: Final commit (if any fixes were needed)**

```bash
git add -A
git commit -m "fix: address build issues from Phase 1 infrastructure migration"
```
