# Phase 1: Infrastructure Migration Design

## Overview

Migrate the foundational infrastructure from field-skill-manage to ai-cockpit: configuration management enhancements and Git sync. This phase provides the base layer that all subsequent phases (version comparison, migration, skillbase) depend on.

**Approach:** Minimal adaptation migration — keep ai-cockpit's existing config model (array-based `AgentConfig[]`), add missing functionality by adapting field-skill-manage's logic to fit the existing architecture.

## Scope

### In Scope
- Git sync service (clone/pull with fallback, repo validation)
- Git sync IPC commands (sync repos, read remote skills)
- Settings commands extensions (custom agent CRUD, import/export config)
- Settings view UI (3 tabs: General, Repositories, Agents)
- Settings store enhancements (new methods for above features)
- RepoPanel enhancement (sync button, status display)

### Out of Scope (later phases)
- Version comparison logic and UI (Phase 2)
- Diff viewer (Phase 2)
- Cross-agent migration (Phase 3)
- Skillbase dependency resolution (Phase 3)
- Operation history with rollback (Phase 3)
- Project list/detail views (Phase 4)
- Search/filter (Phase 4)
- Guide/help page (Phase 4)

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config model | Keep ai-cockpit's existing `AgentConfig[]` array structure | Less disruption, progressive enhancement |
| Agent scope | Keep current 10 built-in agents | Broader coverage than field-skill-manage's 5 |
| Git implementation | git CLI (system git) | Private repo support via existing credentials |
| Settings UI location | Core `src/views/SettingsView.vue` | Settings is a cross-cutting concern, not skills-specific |

## Backend Changes

### New File: `src-tauri/src/services/git_service.rs`

Git operations using system git CLI.

```rust
/// Sync a repository: clone if missing, pull if exists, with fallback strategies.
/// 1. If valid repo → git pull --ff-only
/// 2. If pull fails → git fetch origin + git reset --hard origin/HEAD
/// 3. If all fail → delete and git clone --depth 1
pub fn sync_repo(url: &str, cache_path: &str) -> Result<(), String>

/// Check if a path is a valid git repository
pub fn validate_repo(cache_path: &str) -> bool

/// Get the ISO 8601 timestamp of the latest commit
pub fn get_latest_commit_time(cache_path: &str) -> Result<String, String>
```

Implementation strategy: Adapt from field-skill-manage's `services/git_service.rs`, using `std::process::Command` to invoke system git. Keep the three-tier fallback pattern for resilience.

### New File: `src-tauri/src/commands/git_sync.rs`

IPC command handlers for Git operations.

```rust
#[tauri::command]
pub async fn sync_all_repos(repos: Vec<RepoConfig>) -> Vec<SyncResult>

#[tauri::command]
pub async fn get_remote_skills(repo_id: String, cache_path: String) -> Result<Vec<RemoteSkillInfo>, String>

#[tauri::command]
pub async fn get_remote_skill_detail(repo_id: String, cache_path: String, skill_name: String) -> Result<RemoteSkillDetail, String>
```

`get_remote_skills` scans the cached repo directory:
1. Try parsing `skills.json` manifest first
2. Fallback to scanning directories for `SKILL.md` files

`get_remote_skill_detail` returns full metadata + file list for a specific skill.

### New File: `src-tauri/src/models/git_sync.rs`

```rust
pub struct SyncResult {
    pub repo_id: String,
    pub success: bool,
    pub message: String,
    pub skill_count: u32,
}

pub struct RemoteSkillInfo {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub source_repo: String,
    pub skill_type: String, // "file" | "directory"
}

pub struct RemoteSkillDetail {
    pub info: RemoteSkillInfo,
    pub files: Vec<FileEntry>,
    pub content_hash: String,
}
```

### Extended: `src-tauri/src/commands/settings.rs`

Add these commands to the existing settings command module:

```rust
#[tauri::command]
pub async fn add_custom_agent(
    settings: AppSettings,
    agent_id: String,
    name: String,
    global_path: String,
    project_pattern: String,
) -> Result<AppSettings, String>

#[tauri::command]
pub async fn remove_custom_agent(
    settings: AppSettings,
    agent_id: String,
) -> Result<AppSettings, String>

#[tauri::command]
pub async fn export_config(settings: AppSettings) -> Result<String, String>

#[tauri::command]
pub async fn import_config(json: String) -> Result<AppSettings, String>
```

Portable path handling:
- `export_config`: Replace actual home directory with `${HOME}` in all path strings
- `import_config`: Replace `${HOME}` with actual home directory
- Normalize backslashes to forward slashes for cross-platform compatibility

### Registration in `lib.rs`

Add to `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::git_sync::sync_all_repos,
    commands::git_sync::get_remote_skills,
    commands::git_sync::get_remote_skill_detail,
    commands::settings::add_custom_agent,
    commands::settings::remove_custom_agent,
    commands::settings::export_config,
    commands::settings::import_config,
])
```

## Frontend Changes

### New File: `src/views/SettingsView.vue`

A tabbed settings page using Naive UI's `NTabs` component.

**Tab 1: General**
- Theme toggle (light/dark/system) — `NSwitch`
- Language selector — `NSelect` (zh-CN, en-US)
- Font size slider — `NSlider`
- App version display
- Import/Export config buttons — `NButton`

**Tab 2: Repositories**
- Repository list — card list (consistent with SkillCard pattern) showing: name, URL, enabled status, last sync time
- Add repository — form with name, URL, cache path (auto-derived)
- Edit repository — inline editing
- Delete repository — with confirmation dialog
- Auto-sync toggle — `NSwitch`
- "Sync All" button — triggers `syncAllRepos()`, shows loading spinner and results

**Tab 3: Agents**
- Built-in agents section — list of 10 agents, each showing global path and project pattern (read-only for built-in, or editable paths)
- Custom agents section — add/edit/remove custom agents
  - Add form: agent ID (kebab-case), display name, global path, project pattern
  - Validation: ID must be unique, paths must be valid

### Extended: `src/plugins/settings/store.ts`

New methods on the existing Pinia store:

```typescript
// Custom agent management
async addCustomAgent(agent: { id: string; name: string; globalPath: string; projectPattern: string }): Promise<void>
async removeCustomAgent(agentId: string): Promise<void>

// Git sync
async syncAllRepos(): Promise<SyncResult[]>
async getRemoteSkills(repoId: string): Promise<RemoteSkillInfo[]>

// Config portability
async exportConfig(): Promise<string>
async importConfig(json: string): Promise<void>
```

### Enhanced: `src/plugins/skills/components/RepoPanel.vue`

Add to the existing RepoPanel:
- "Sync" button next to each repo → calls `syncAllRepos()`
- Sync status indicator: loading spinner, success/fail message, skill count
- Last sync timestamp display (from `get_latest_commit_time`)
- Refresh remote skills list after successful sync

### Extended: `src/plugins/skills/types.ts`

Add types for new features:

```typescript
interface SyncResult {
  repoId: string
  success: boolean
  message: string
  skillCount: number
}

interface RemoteSkillInfo {
  name: string
  description: string
  version?: string
  sourceRepo: string
  skillType: 'file' | 'directory'
}

interface RemoteSkillDetail {
  info: RemoteSkillInfo
  files: FileEntry[]
  contentHash: string
}
```

## Data Flow

### Git Sync Flow

```
User clicks "Sync All" in RepoPanel/SettingsView
  → settings.store.syncAllRepos()
    → invoke('sync_all_repos', { repos })
      → git_sync.rs: iterate repos, call git_service::sync_repo for each
        → git_service: git pull --ff-only
          → if fails: git fetch + reset --hard
          → if fails: delete + clone --depth 1
      → return Vec<SyncResult>
    → update store with results
    → if success: invoke('get_remote_skills') to refresh skill list
```

### Custom Agent Flow

```
User fills add-agent form in SettingsView
  → settings.store.addCustomAgent({ id, name, globalPath, projectPattern })
    → invoke('add_custom_agent', { settings, agent_id, name, global_path, project_pattern })
      → settings.rs: validate ID uniqueness, add to settings.agents
      → return updated settings
    → update store
```

### Config Export/Import Flow

```
Export:
  → settings.store.exportConfig()
    → invoke('export_config', { settings })
      → settings.rs: serialize to JSON, replace home dir with ${HOME}
      → return JSON string
    → trigger browser download / save dialog

Import:
  → User selects file
  → settings.store.importConfig(json)
    → invoke('import_config', { json })
      → settings.rs: parse JSON, replace ${HOME} with actual home dir, validate
      → return AppSettings
    → update store
```

## Error Handling

- **Git sync failures**: Return per-repo SyncResult with error message. UI shows which repos failed and why. Partial success supported (some repos sync, others don't).
- **Custom agent validation**: Backend validates uniqueness, prevents overriding built-in agents. Frontend shows validation errors inline.
- **Config import validation**: Backend validates JSON structure, required fields, path formats. Invalid configs rejected with specific error messages.
- **Network errors**: Git operations timeout after 30s. UI shows connection error, suggests checking network or credentials.

## Testing Strategy

- **Unit tests**: Rust services (git_service, settings commands) get unit tests
- **Integration tests**: IPC command end-to-end tests via Tauri test framework
- **E2E tests**: Settings page interactions (add/remove repo, add custom agent, sync) via WDIO
- **Manual testing**: Private repo sync with SSH authentication

## Migration Reference

Primary reference files from field-skill-manage:

| Feature | Source File | Lines |
|---------|------------|-------|
| Git sync logic | `services/git_service.rs` | ~150 |
| Git commands | `commands/git_sync.rs` | ~80 |
| Config commands | `commands/config.rs` | ~200 |
| Config models | `models/config.rs` | ~150 |
| Settings UI | `views/SettingsView.vue` | ~600 |
| Config store | `stores/config.ts` | ~200 |

Adaptation notes:
- field-skill-manage uses `AppState` Mutex for config; ai-cockpit passes settings as parameters → adapt command signatures
- field-skill-manage uses `HashMap<AgentId, Path>`; ai-cockpit uses `AgentConfig[]` → convert access patterns
- field-skill-manage has 5 built-in agents; ai-cockpit has 10 → keep ai-cockpit's list
