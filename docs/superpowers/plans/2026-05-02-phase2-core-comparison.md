# Phase 2: Core Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement local-vs-remote skill comparison, file-level diff, and skill preview — the core "compare and update" workflow.

**Architecture:** Backend Rust handles skill matching (by name) and hash-based comparison. Frontend receives `SkillComparison[]` with status (Same/Outdated/LocalOnly/RemoteOnly), then loads file-level diffs and line-level diffs on demand. Three new Vue components integrate into the existing SkillsMainView as a comparison mode toggle.

**Tech Stack:** Rust (sha2, existing skills_service), Vue 3 + Naive UI, `diff` npm package (line diffing), `marked` + `highlight.js` (preview rendering)

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `src/plugins/skills/components/SkillCompareTable.vue` | Comparison table with status badges, batch actions |
| `src/plugins/skills/components/SkillDiffViewer.vue` | Modal showing file-level + line-level diff |
| `src/plugins/skills/components/SkillPreviewModal.vue` | Modal with file tree + content preview |

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/src/models/skills.rs` | Add `SkillComparison`, `ComparisonStatus`, `FileDiffEntry`, `DiffFileContent` |
| `src-tauri/src/services/skills_service.rs` | Add `build_skill_comparisons()`, `build_skill_diff()`, `get_diff_file_content()` |
| `src-tauri/src/commands/skills.rs` | Add `compare_skills`, `get_skill_diff`, `get_diff_file_content` commands |
| `src-tauri/src/lib.rs` | Register 3 new commands |
| `src/plugins/skills/types.ts` | Add `SkillComparison`, `ComparisonStatus`, `FileDiffEntry`, `DiffFileContent`, `DiffLine` |
| `src/plugins/skills/store.ts` | Add comparison/diff state and methods |
| `src/plugins/skills/views/SkillsMainView.vue` | Add comparison mode toggle + integrate SkillCompareTable |
| `src/plugins/skills/i18n/zh-CN.json` | Add comparison/diff/preview i18n keys |
| `src/plugins/skills/i18n/en-US.json` | Add comparison/diff/preview i18n keys |
| `package.json` | Add `diff`, `marked`, `highlight.js` dependencies |

---

## Task 1: Backend — Comparison & Diff Models

**Files:**
- Modify: `src-tauri/src/models/skills.rs`

- [ ] **Step 1: Add new model types to `models/skills.rs`**

Append these types after the existing `ProjectDetail` struct (after line 155):

```rust
/// Comparison status between local and remote skill versions
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonStatus {
    Same,
    Outdated,
    LocalOnly,
    RemoteOnly,
}

/// A single skill comparison result pairing local and remote versions
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillComparison {
    pub name: String,
    pub status: ComparisonStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<SkillInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<SkillInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repo: Option<String>,
}

/// File-level diff entry between local and remote
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffEntry {
    pub path: String,
    pub file_name: String,
    pub diff_type: DiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_size: Option<u64>,
}

/// File content for line-by-line diff
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffFileContent {
    pub local_content: Option<String>,
    pub remote_content: Option<String>,
}

/// Full skill diff result containing all file-level diffs
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiffResult {
    pub skill_name: String,
    pub file_diffs: Vec<FileDiffEntry>,
    pub added_count: u32,
    pub removed_count: u32,
    pub modified_count: u32,
    pub unchanged_count: u32,
}
```

- [ ] **Step 2: Build and verify models compile**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models/skills.rs
git commit -m "feat(skills): add comparison and diff model types"
```

---

## Task 2: Backend — Comparison Service Logic

**Files:**
- Modify: `src-tauri/src/services/skills_service.rs`

- [ ] **Step 1: Add `build_skill_comparisons()` function**

Append to `skills_service.rs` (before the `#[cfg(test)]` block):

```rust
/// Build skill comparisons by pairing local skills with remote skills.
///
/// Scans local directory for installed skills, loads remote skills from all
/// enabled repos, pairs by name, and determines comparison status via hash.
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

    // 2. Scan remote skills from all repos, build map of name → (SkillInfo, repo_id)
    let mut remote_map: HashMap<String, (SkillInfo, String)> = HashMap::new();
    for (repo_id, cache_path) in repos {
        if let Ok(remote_result) = scan_remote_skills(cache_path, repo_id) {
            for skill in remote_result.skills {
                // Keep the first found remote version (first repo wins)
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
```

Add the missing import at the top of the file:

```rust
use crate::models::skills::*;
```

(Replace the existing `use crate::models::skills::*;` — it's already there, but verify `ComparisonStatus` and `SkillComparison` are available.)

- [ ] **Step 2: Add `build_skill_diff()` function**

```rust
/// Build file-level diff between local and remote skill directories.
/// Compares each file by SHA256 hash to determine status.
pub fn build_skill_diff(
    local_path: &str,
    remote_path: &str,
) -> Result<SkillDiffResult, String> {
    let local = Path::new(local_path);
    let remote = Path::new(remote_path);

    // Collect all relative file paths from both sides
    let mut local_files: HashMap<String, (String, u64)> = HashMap::new(); // rel_path → (hash, size)
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

            // Normalize path separators to forward slashes
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
```

- [ ] **Step 3: Add `get_diff_file_content()` function**

```rust
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
```

- [ ] **Step 4: Build and verify service compiles**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/skills_service.rs
git commit -m "feat(skills): add comparison and diff service logic"
```

---

## Task 3: Backend — IPC Commands + Registration

**Files:**
- Modify: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add three new commands to `commands/skills.rs`**

Append after the existing `verify_skill_integrity` command:

```rust
use crate::models::skills::{SkillComparison, SkillDiffResult, DiffFileContent};
use crate::services::settings_service::RepoConfig;

/// Compare local skills against remote repos.
/// Returns a comparison list pairing local and remote skills by name.
#[tauri::command]
pub async fn compare_skills(
    agent_id: String,
    scope: String,
    global_path: String,
    project_path: String,
    project_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<Vec<SkillComparison>, String> {
    let expanded = expand_path(&global_path);

    // Determine local scan directory
    let local_dir = if scope == "project" {
        if project_dir.is_empty() {
            return Ok(vec![]);
        }
        format!("{}/{}", project_dir, project_path)
    } else {
        format!("{}/skills", expanded)
    };

    // Collect enabled repos with resolved cache paths
    let repo_dirs: Vec<(String, String)> = repos
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| {
            let cache = if r.cache_path.is_empty() {
                dirs::data_dir()
                    .map(|d| d.join("ai-cockpit").join("repos").join(&r.id))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else if r.cache_path.starts_with('~') {
                let home = dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                r.cache_path.replacen("~", &home, 1)
            } else {
                r.cache_path.clone()
            };
            (r.id, cache)
        })
        .collect();

    skills_service::build_skill_comparisons(&local_dir, &repo_dirs)
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
```

Also add the `RepoConfig` import at the top (it needs `use crate::services::settings_service::RepoConfig;`).

- [ ] **Step 2: Register commands in `lib.rs`**

Add to the `invoke_handler` macro, after `verify_skill_integrity`:

```rust
            // Skills — Batch 3: Comparison
            commands::skills::compare_skills,
            commands::skills::get_skill_diff,
            commands::skills::get_diff_file_content,
```

- [ ] **Step 3: Build and verify commands compile**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/skills.rs src-tauri/src/lib.rs
git commit -m "feat(skills): add compare_skills, get_skill_diff, get_diff_file_content IPC commands"
```

---

## Task 4: Frontend — Install NPM Dependencies

**Files:**
- Modify: `package.json`

- [ ] **Step 1: Install diff, marked, and highlight.js**

Run:
```bash
cd "D:\Project\ai-cockpit\.claude\worktrees\vibrant-fermi-029bf7" && npm install diff marked highlight.js
```

Then install TypeScript types:
```bash
cd "D:\Project\ai-cockpit\.claude\worktrees\vibrant-fermi-029bf7" && npm install -D @types/diff
```

- [ ] **Step 2: Verify installation**

Run: `cat package.json | grep -E '"(diff|marked|highlight)"'`
Expected: Three entries in dependencies

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: add diff, marked, highlight.js dependencies for comparison UI"
```

---

## Task 5: Frontend — Comparison Types

**Files:**
- Modify: `src/plugins/skills/types.ts`

- [ ] **Step 1: Add comparison and diff types**

Append after the existing `RemoteSkillDetail` interface:

```typescript
/** Status of a skill comparison between local and remote */
export type ComparisonStatus = 'same' | 'outdated' | 'localOnly' | 'remoteOnly'

/** A comparison pairing local and remote skill info */
export interface SkillComparison {
  name: string
  status: ComparisonStatus
  local?: SkillInfo
  remote?: SkillInfo
  sourceRepo?: string
}

/** File-level diff entry */
export interface FileDiffEntry {
  path: string
  fileName: string
  diffType: DiffStatus
  localSize?: number
  remoteSize?: number
}

/** Full skill diff result */
export interface SkillDiffResult {
  skillName: string
  fileDiffs: FileDiffEntry[]
  addedCount: number
  removedCount: number
  modifiedCount: number
  unchangedCount: number
}

/** File content for line-by-line diff */
export interface DiffFileContent {
  localContent?: string
  remoteContent?: string
}

/** A single line in a diff output */
export interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  oldLineNumber?: number
  newLineNumber?: number
  content: string
}
```

- [ ] **Step 2: Commit**

```bash
git add src/plugins/skills/types.ts
git commit -m "feat(skills): add comparison and diff frontend types"
```

---

## Task 6: Frontend — Store Enhancements

**Files:**
- Modify: `src/plugins/skills/store.ts`

- [ ] **Step 1: Add comparison state and methods**

Add new imports at top:

```typescript
import type {
  SkillInfo,
  SkillScope,
  ScanResult,
  OperationResult,
  SkillOperation,
  SkillComparison,
  SkillDiffResult,
  DiffFileContent,
} from "./types";
```

Add new state refs inside the `defineStore` function (after `error`):

```typescript
  const comparisons = ref<SkillComparison[]>([]);
  const comparisonMode = ref(false);
  const currentDiff = ref<SkillDiffResult | null>(null);
  const loadingDiff = ref(false);
```

Add computed for comparison counts:

```typescript
  const comparisonCounts = computed(() => {
    const counts = { outdated: 0, remoteOnly: 0, localOnly: 0, same: 0 };
    for (const c of comparisons.value) {
      counts[c.status]++;
    }
    return counts;
  });
```

Add new methods:

```typescript
  async function loadComparisons(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const settingsStore = useSettingsStore();
      const agent = settingsStore.agents.find((a) => a.id === currentAgentId.value);
      if (!agent) return;

      comparisons.value = await invoke<SkillComparison[]>("compare_skills", {
        agentId: currentAgentId.value,
        scope: currentScope.value,
        globalPath: agent.globalPath,
        projectPath: agent.projectPath,
        projectDir: currentProjectPath.value,
        repos: settingsStore.repos,
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      console.error("[SkillsStore] Failed to load comparisons:", e);
    } finally {
      loading.value = false;
    }
  }

  function toggleComparisonMode(): void {
    comparisonMode.value = !comparisonMode.value;
    if (comparisonMode.value) {
      loadComparisons();
    }
  }

  async function loadSkillDiff(
    localPath: string,
    remotePath: string,
  ): Promise<SkillDiffResult> {
    loadingDiff.value = true;
    try {
      const result = await invoke<SkillDiffResult>("get_skill_diff", {
        localSkillPath: localPath,
        remoteSkillPath: remotePath,
      });
      currentDiff.value = result;
      return result;
    } finally {
      loadingDiff.value = false;
    }
  }

  async function loadDiffFileContent(
    localSkillPath: string,
    remoteSkillPath: string,
    relFilePath: string,
  ): Promise<DiffFileContent> {
    return invoke<DiffFileContent>("get_diff_file_content", {
      localSkillPath,
      remoteSkillPath,
      relFilePath,
    });
  }

  function clearDiff(): void {
    currentDiff.value = null;
  }
```

Update the `switchAgent` method to clear comparisons:

```typescript
  async function switchAgent(agentId: string): Promise<void> {
    currentAgentId.value = agentId;
    selectedSkills.value.clear();
    comparisons.value = [];

    const skillsMap =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    if (!skillsMap.has(agentId)) {
      await scanSkills(agentId, currentScope.value);
    }
    if (comparisonMode.value) {
      await loadComparisons();
    }
  }
```

Update the `switchScope` method similarly:

```typescript
  async function switchScope(scope: SkillScope): Promise<void> {
    currentScope.value = scope;
    selectedSkills.value.clear();
    comparisons.value = [];

    const skillsMap =
      scope === "global" ? globalSkills.value : projectSkills.value;
    if (!skillsMap.has(currentAgentId.value)) {
      await scanSkills(currentAgentId.value, scope);
    }
    if (comparisonMode.value) {
      await loadComparisons();
    }
  }
```

Add to the return statement:

```typescript
    // Comparison
    comparisons,
    comparisonMode,
    comparisonCounts,
    currentDiff,
    loadingDiff,
    loadComparisons,
    toggleComparisonMode,
    loadSkillDiff,
    loadDiffFileContent,
    clearDiff,
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npx vue-tsc --noEmit 2>&1 | tail -10`
Expected: No errors related to the store

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/store.ts
git commit -m "feat(skills): add comparison/diff state and methods to store"
```

---

## Task 7: Frontend — SkillCompareTable Component

**Files:**
- Create: `src/plugins/skills/components/SkillCompareTable.vue`

- [ ] **Step 1: Create SkillCompareTable.vue**

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import {
  NDataTable,
  NSpace,
  NButton,
  NTag,
  NTooltip,
  useDialog,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import { useSettingsStore } from "@/plugins/settings/store";
import type { SkillComparison, SkillInfo } from "../types";
import type { DataTableColumns } from "naive-ui";

const { t } = useI18n();
const store = useSkillsStore();
const settingsStore = useSettingsStore();
const dialog = useDialog();

const emit = defineEmits<{
  diff: [localPath: string, remotePath: string];
  preview: [skillPath: string, skillName: string];
}>();

const selectedRowKeys = ref<string[]>([]);
const operatingKeys = ref<Set<string>>(new Set());

const columns = computed<DataTableColumns<SkillComparison>>(() => [
  {
    type: "selection",
    options: ["all", "none"],
  },
  {
    title: t("skills.compare.status"),
    key: "status",
    width: 100,
    render(row) {
      const config: Record<string, { type: "success" | "warning" | "info" | "error"; label: string }> = {
        same: { type: "success", label: t("skills.compare.statusSame") },
        outdated: { type: "warning", label: t("skills.compare.statusOutdated") },
        localOnly: { type: "info", label: t("skills.compare.statusLocalOnly") },
        remoteOnly: { type: "error", label: t("skills.compare.statusRemoteOnly") },
      };
      const c = config[row.status];
      return h(NTag, { type: c.type, size: "small" }, { default: () => c.label });
    },
  },
  {
    title: t("skills.compare.name"),
    key: "name",
    ellipsis: { tooltip: true },
    render(row) {
      return row.local?.meta?.name ?? row.remote?.meta?.name ?? row.name;
    },
  },
  {
    title: t("skills.compare.sourceRepo"),
    key: "sourceRepo",
    width: 120,
    ellipsis: { tooltip: true },
    render(row) {
      return row.sourceRepo ?? "-";
    },
  },
  {
    title: t("skills.compare.localVersion"),
    key: "localVersion",
    width: 90,
    render(row) {
      if (!row.local) return h("span", { style: "color: #999" }, "-");
      return row.local.meta?.version ? `v${row.local.meta.version}` : "-";
    },
  },
  {
    title: t("skills.compare.remoteVersion"),
    key: "remoteVersion",
    width: 100,
    render(row) {
      if (!row.remote) return h("span", { style: "color: #999" }, "-");
      return row.remote.meta?.version ? `v${row.remote.meta.version}` : "-";
    },
  },
  {
    title: t("skills.compare.description"),
    key: "description",
    ellipsis: { tooltip: true },
    render(row) {
      return row.remote?.meta?.description ?? row.local?.meta?.description ?? "-";
    },
  },
  {
    title: t("skills.compare.actions"),
    key: "actions",
    width: 180,
    fixed: "right",
    render(row) {
      const buttons = [];

      if (row.status === "remoteOnly" && row.remote) {
        buttons.push(
          h(NButton, {
            size: "tiny",
            type: "primary",
            loading: operatingKeys.value.has(row.name),
            onClick: () => handleInstall(row),
          }, { default: () => t("skills.actions.install") })
        );
      }

      if (row.status === "outdated") {
        buttons.push(
          h(NButton, {
            size: "tiny",
            quaternary: true,
            onClick: () => emit("diff", row.local!.path, row.remote!.path),
          }, { default: () => t("skills.compare.viewDiff") })
        );
        buttons.push(
          h(NButton, {
            size: "tiny",
            type: "warning",
            loading: operatingKeys.value.has(row.name),
            onClick: () => handleUpdate(row),
          }, { default: () => t("skills.actions.update") })
        );
      }

      if (row.status === "same") {
        buttons.push(
          h(NTooltip, null, {
            trigger: () => h(NButton, {
              size: "tiny",
              quaternary: true,
              onClick: () => handleReinstall(row),
            }, { default: () => t("skills.compare.reinstall") }),
            default: () => t("skills.compare.reinstallTip"),
          })
        );
      }

      if (row.status === "localOnly" && row.local) {
        buttons.push(
          h(NButton, {
            size: "tiny",
            type: "error",
            quaternary: true,
            onClick: () => handleUninstall(row),
          }, { default: () => t("skills.actions.uninstall") })
        );
      }

      // Preview button for any skill with a remote version
      if (row.remote) {
        buttons.push(
          h(NButton, {
            size: "tiny",
            quaternary: true,
            onClick: () => emit("preview", row.remote!.path, row.name),
          }, { default: () => t("skills.compare.preview") })
        );
      }

      return h(NSpace, { size: 4, wrap: false }, { default: () => buttons });
    },
  },
]);

import { h } from "vue";

const rowKey = (row: SkillComparison) => row.name;

function handleRowSelect(keys: string[]) {
  selectedRowKeys.value = keys;
}

async function handleInstall(comparison: SkillComparison) {
  if (!comparison.remote) return;
  const agent = store.getCurrentAgentConfig();
  if (!agent) return;
  operatingKeys.value.add(comparison.name);
  try {
    const targetPath = `${agent.globalPath}/skills/${comparison.name}`;
    const home = await getHome();
    await store.installSkill(comparison.remote.path, `${home}/${targetPath}`);
    await store.loadComparisons();
  } finally {
    operatingKeys.value.delete(comparison.name);
  }
}

async function handleUpdate(comparison: SkillComparison) {
  if (!comparison.local || !comparison.remote) return;
  operatingKeys.value.add(comparison.name);
  try {
    await store.updateSkill(comparison.remote.path, comparison.local.path);
    await store.loadComparisons();
  } finally {
    operatingKeys.value.delete(comparison.name);
  }
}

async function handleReinstall(comparison: SkillComparison) {
  if (!comparison.local || !comparison.remote) return;
  operatingKeys.value.add(comparison.name);
  try {
    await store.updateSkill(comparison.remote.path, comparison.local.path);
    await store.loadComparisons();
  } finally {
    operatingKeys.value.delete(comparison.name);
  }
}

function handleUninstall(comparison: SkillComparison) {
  if (!comparison.local) return;
  dialog.warning({
    title: t("skills.confirm.uninstall"),
    content: t("skills.confirm.uninstallMsg", { name: comparison.name }),
    positiveText: t("skills.actions.uninstall"),
    negativeText: t("skills.repos.cancel"),
    onPositiveClick: async () => {
      await store.uninstallSkill(comparison.local!.path);
      await store.loadComparisons();
    },
  });
}

async function handleBatchInstall() {
  const names = selectedRowKeys.value;
  if (names.length === 0) return;

  const agent = store.getCurrentAgentConfig();
  if (!agent) return;
  const home = await getHome();

  const operations = names
    .map((name) => {
      const comp = store.comparisons.find((c) => c.name === name);
      if (comp?.remote && (comp.status === "remoteOnly" || comp.status === "outdated")) {
        const targetPath = comp.local?.path ?? `${home}/${agent.globalPath}/skills/${name}`;
        return { operationType: "install" as const, source: comp.remote.path, targetPath };
      }
      return null;
    })
    .filter(Boolean);

  if (operations.length === 0) return;

  await store.batchOperate(operations);
  await store.loadComparisons();
  selectedRowKeys.value = [];
}

async function handleBatchUpdate() {
  const names = selectedRowKeys.value;
  if (names.length === 0) return;

  const operations = names
    .map((name) => {
      const comp = store.comparisons.find((c) => c.name === name);
      if (comp?.local && comp?.remote && comp.status === "outdated") {
        return { operationType: "update" as const, source: comp.remote.path, targetPath: comp.local.path };
      }
      return null;
    })
    .filter(Boolean);

  if (operations.length === 0) return;

  await store.batchOperate(operations);
  await store.loadComparisons();
  selectedRowKeys.value = [];
}

async function getHome(): Promise<string> {
  try {
    return await invoke<string>("get_data_dir") + "/..";
  } catch {
    return "";
  }
}

import { invoke } from "@tauri-apps/api/core";

const canBatchInstall = computed(() =>
  selectedRowKeys.value.some((name) => {
    const comp = store.comparisons.find((c) => c.name === name);
    return comp?.status === "remoteOnly" || comp?.status === "outdated";
  })
);

const canBatchUpdate = computed(() =>
  selectedRowKeys.value.some((name) => {
    const comp = store.comparisons.find((c) => c.name === name);
    return comp?.status === "outdated";
  })
);
</script>

<template>
  <div>
    <NSpace v-if="selectedRowKeys.length > 0" justify="end" style="margin-bottom: 12px">
      <NButton size="small" type="primary" :disabled="!canBatchInstall" @click="handleBatchInstall">
        {{ t("skills.compare.batchInstall") }}
      </NButton>
      <NButton size="small" type="warning" :disabled="!canBatchUpdate" @click="handleBatchUpdate">
        {{ t("skills.compare.batchUpdate") }}
      </NButton>
    </NSpace>

    <NDataTable
      :columns="columns"
      :data="store.comparisons"
      :row-key="rowKey"
      :checked-row-keys="selectedRowKeys"
      @update:checked-row-keys="handleRowSelect"
      :loading="store.loading"
      :scroll-x="900"
      :pagination="{ pageSize: 20 }"
      size="small"
      striped
    />
  </div>
</template>
```

- [ ] **Step 2: Verify component compiles**

Run: `npx vue-tsc --noEmit 2>&1 | grep -i "SkillCompareTable" || echo "No errors for SkillCompareTable"`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/components/SkillCompareTable.vue
git commit -m "feat(skills): add SkillCompareTable component with status, actions, batch ops"
```

---

## Task 8: Frontend — SkillDiffViewer Component

**Files:**
- Create: `src/plugins/skills/components/SkillDiffViewer.vue`

- [ ] **Step 1: Create SkillDiffViewer.vue**

```vue
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
  NModal,
  NCard,
  NDataTable,
  NSpace,
  NTag,
  NText,
  NButton,
  NScrollbar,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import type { SkillDiffResult, FileDiffEntry, DiffFileContent, DiffLine } from "../types";
import type { DataTableColumns } from "naive-ui";
import { diffLines } from "diff";
import { h } from "vue";

const props = defineProps<{
  show: boolean;
  localPath: string;
  remotePath: string;
}>();

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const store = useSkillsStore();

const diffResult = ref<SkillDiffResult | null>(null);
const selectedFile = ref<string | null>(null);
const fileDiffLines = ref<DiffLine[]>([]);
const loadingContent = ref(false);

const fileColumns = computed<DataTableColumns<FileDiffEntry>>(() => [
  {
    title: t("skills.diff.file"),
    key: "fileName",
    ellipsis: { tooltip: true },
    render(row) {
      return h(NSpace, { align: "center", size: 4 }, {
        default: () => [
          h(NTag, {
            type: row.diffType === "added" ? "success" : row.diffType === "removed" ? "error" : row.diffType === "modified" ? "warning" : "default",
            size: "small",
          }, {
            default: () => row.diffType === "same" ? "=" : row.diffType === "added" ? "+" : row.diffType === "removed" ? "-" : "~",
          }),
          h("span", null, row.path),
        ],
      });
    },
  },
  {
    title: t("skills.diff.status"),
    key: "diffType",
    width: 90,
    render(row) {
      const map: Record<string, { type: "success" | "error" | "warning" | "default"; label: string }> = {
        added: { type: "success", label: t("skills.diff.added") },
        removed: { type: "error", label: t("skills.diff.removed") },
        modified: { type: "warning", label: t("skills.diff.modified") },
        same: { type: "default", label: t("skills.diff.unchanged") },
      };
      const c = map[row.diffType];
      return h(NTag, { type: c.type, size: "small" }, { default: () => c.label });
    },
  },
]);

function handleFileClick(row: FileDiffEntry) {
  if (row.diffType === "same") return;
  selectedFile.value = row.path;
  loadFileDiff(row.path);
}

async function loadFileDiff(relPath: string) {
  loadingContent.value = true;
  fileDiffLines.value = [];
  try {
    const content: DiffFileContent = await store.loadDiffFileContent(
      props.localPath,
      props.remotePath,
      relPath,
    );
    computeLineDiff(content);
  } catch (e) {
    console.error("[SkillDiffViewer] Failed to load file content:", e);
  } finally {
    loadingContent.value = false;
  }
}

function computeLineDiff(content: DiffFileContent) {
  const local = content.localContent ?? "";
  const remote = content.remoteContent ?? "";

  const changes = diffLines(remote, local);
  const lines: DiffLine[] = [];
  let oldLine = 1;
  let newLine = 1;

  for (const change of changes) {
    const text = change.value;
    const lineCount = text.split("\n").length - (text.endsWith("\n") ? 1 : 0);

    if (change.added) {
      for (let i = 0; i < lineCount; i++) {
        lines.push({
          type: "added",
          newLineNumber: newLine++,
          content: text.split("\n")[i],
        });
      }
    } else if (change.removed) {
      for (let i = 0; i < lineCount; i++) {
        lines.push({
          type: "removed",
          oldLineNumber: oldLine++,
          content: text.split("\n")[i],
        });
      }
    } else {
      for (let i = 0; i < lineCount; i++) {
        lines.push({
          type: "unchanged",
          oldLineNumber: oldLine++,
          newLineNumber: newLine++,
          content: text.split("\n")[i],
        });
      }
    }
  }

  fileDiffLines.value = lines;
}

watch(() => props.show, async (show) => {
  if (show && props.localPath && props.remotePath) {
    diffResult.value = await store.loadSkillDiff(props.localPath, props.remotePath);
    selectedFile.value = null;
    fileDiffLines.value = [];
  }
});

function getLineClass(line: DiffLine): string {
  if (line.type === "added") return "diff-line-added";
  if (line.type === "removed") return "diff-line-removed";
  return "diff-line-unchanged";
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :title="t('skills.diff.title', { name: diffResult?.skillName ?? '' })"
    style="width: 90vw; max-width: 1000px"
    :mask-closable="true"
    @update:show="(v: boolean) => !v && emit('close')"
  >
    <template v-if="diffResult">
      <NSpace :size="12" style="margin-bottom: 16px">
        <NTag type="success" size="small">{{ t("skills.diff.added") }}: {{ diffResult.addedCount }}</NTag>
        <NTag type="error" size="small">{{ t("skills.diff.removed") }}: {{ diffResult.removedCount }}</NTag>
        <NTag type="warning" size="small">{{ t("skills.diff.modified") }}: {{ diffResult.modifiedCount }}</NTag>
        <NTag size="small">{{ t("skills.diff.unchanged") }}: {{ diffResult.unchangedCount }}</NTag>
      </NSpace>

      <div style="display: flex; gap: 16px; height: 500px">
        <div style="width: 280px; flex-shrink: 0; overflow: auto; border: 1px solid var(--n-border-color); border-radius: 4px">
          <NDataTable
            :columns="fileColumns"
            :data="diffResult.fileDiffs"
            :row-props="(row: FileDiffEntry) => ({
              style: row.diffType !== 'same' ? 'cursor: pointer' : 'opacity: 0.6',
              onClick: () => handleFileClick(row),
            })"
            size="tiny"
            :pagination="false"
          />
        </div>

        <div v-if="selectedFile" style="flex: 1; overflow: hidden; display: flex; flex-direction: column">
          <NText strong style="margin-bottom: 8px; display: block">{{ selectedFile }}</NText>
          <NScrollbar style="flex: 1">
            <div v-if="loadingContent" style="padding: 16px; text-align: center; color: #999">
              {{ t("skills.diff.loading") }}
            </div>
            <table v-else class="diff-table">
              <tr v-for="(line, idx) in fileDiffLines" :key="idx" :class="getLineClass(line)">
                <td class="diff-line-num">{{ line.oldLineNumber ?? "" }}</td>
                <td class="diff-line-num">{{ line.newLineNumber ?? "" }}</td>
                <td class="diff-prefix">{{ line.type === "added" ? "+" : line.type === "removed" ? "-" : " " }}</td>
                <td class="diff-content"><pre>{{ line.content }}</pre></td>
              </tr>
            </table>
          </NScrollbar>
        </div>

        <div v-else style="flex: 1; display: flex; align-items: center; justify-content: center; color: #999">
          {{ t("skills.diff.selectFile") }}
        </div>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.diff-table {
  width: 100%;
  border-collapse: collapse;
  font-family: "Cascadia Code", "Fira Code", "Consolas", monospace;
  font-size: 13px;
  line-height: 1.5;
}
.diff-line-num {
  width: 40px;
  text-align: right;
  padding: 0 8px;
  color: #999;
  background: var(--n-color-embedded);
  user-select: none;
  vertical-align: top;
}
.diff-prefix {
  width: 20px;
  text-align: center;
  padding: 0 4px;
  user-select: none;
  vertical-align: top;
}
.diff-content {
  padding: 0 8px;
  white-space: pre-wrap;
  word-break: break-all;
  vertical-align: top;
}
.diff-content pre {
  margin: 0;
  white-space: pre-wrap;
}
.diff-line-added { background: rgba(46, 160, 67, 0.15); }
.diff-line-removed { background: rgba(248, 81, 73, 0.15); }
.diff-line-unchanged { }
</style>
```

- [ ] **Step 2: Verify component compiles**

Run: `npx vue-tsc --noEmit 2>&1 | grep -i "SkillDiffViewer" || echo "No errors for SkillDiffViewer"`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/components/SkillDiffViewer.vue
git commit -m "feat(skills): add SkillDiffViewer with file-level and line-level diff"
```

---

## Task 9: Frontend — SkillPreviewModal Component

**Files:**
- Create: `src/plugins/skills/components/SkillPreviewModal.vue`

- [ ] **Step 1: Create SkillPreviewModal.vue**

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
import {
  NModal,
  NSpace,
  NText,
  NTag,
  NScrollbar,
  NSpin,
  NTree,
  NDivider,
} from "naive-ui";
import type { TreeOption } from "naive-ui";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import type { FileEntry } from "../types";
import { marked } from "marked";
import hljs from "highlight.js";

const props = defineProps<{
  show: boolean;
  skillPath: string;
  skillName: string;
}>();

const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

const files = ref<FileEntry[]>([]);
const selectedFile = ref<string | null>(null);
const fileContent = ref<string>("");
const renderedContent = ref<string>("");
const loading = ref(false);
const isMarkdown = ref(false);

watch(() => props.show, async (show) => {
  if (show && props.skillPath) {
    loading.value = true;
    try {
      files.value = await invoke<FileEntry[]>("get_skill_file_tree", {
        skillPath: props.skillPath,
      });
      // Auto-select SKILL.md if it exists
      const skillMd = findSkillMd(files.value);
      if (skillMd) {
        await loadFile(skillMd);
      }
    } catch (e) {
      console.error("[SkillPreviewModal] Failed to load file tree:", e);
    } finally {
      loading.value = false;
    }
  }
});

function findSkillMd(entries: FileEntry[]): string | null {
  for (const entry of entries) {
    if (entry.name === "SKILL.md" && !entry.isDir) return entry.path;
    if (entry.children.length > 0) {
      const found = findSkillMd(entry.children);
      if (found) return found;
    }
  }
  return null;
}

function toTreeOptions(entries: FileEntry[]): TreeOption[] {
  return entries.map((entry) => ({
    key: entry.path,
    label: entry.name,
    prefix: entry.isDir ? "📁" : getFileIcon(entry.name),
    children: entry.children.length > 0 ? toTreeOptions(entry.children) : undefined,
    isLeaf: !entry.isDir,
  }));
}

function getFileIcon(name: string): string {
  if (name.endsWith(".md")) return "📄";
  if (name.endsWith(".ts") || name.endsWith(".js")) return "📜";
  if (name.endsWith(".json")) return "📋";
  if (name.endsWith(".rs")) return "⚙️";
  if (name.endsWith(".yaml") || name.endsWith(".yml")) return "📝";
  return "📄";
}

async function handleSelect(keys: string[]) {
  if (keys.length > 0) {
    await loadFile(keys[0]);
  }
}

async function loadFile(filePath: string) {
  selectedFile.value = filePath;
  loading.value = true;
  try {
    const content = await invoke<string>("read_skill_file", { filePath });
    fileContent.value = content;

    const fileName = filePath.split(/[/\\]/).pop() ?? "";
    isMarkdown.value = fileName.endsWith(".md");

    if (isMarkdown.value) {
      renderedContent.value = await marked(content, {
        gfm: true,
        breaks: true,
      });
    } else {
      // Try syntax highlighting for non-markdown files
      try {
        const ext = fileName.split(".").pop() ?? "";
        const langMap: Record<string, string> = {
          ts: "typescript",
          tsx: "typescript",
          js: "javascript",
          json: "json",
          rs: "rust",
          yaml: "yaml",
          yml: "yaml",
          py: "python",
          vue: "html",
        };
        const lang = langMap[ext] ?? ext;
        const result = hljs.highlight(content, { language: lang });
        renderedContent.value = `<pre><code class="hljs">${result.value}</code></pre>`;
      } catch {
        renderedContent.value = `<pre>${escapeHtml(content)}</pre>`;
      }
    }
  } catch (e) {
    fileContent.value = "";
    renderedContent.value = `<p style="color: #f00">${t("skills.preview.loadError")}: ${e}</p>`;
  } finally {
    loading.value = false;
  }
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :title="t('skills.preview.title', { name: skillName })"
    style="width: 90vw; max-width: 1000px; height: 80vh"
    :mask-closable="true"
    @update:show="(v: boolean) => !v && emit('close')"
  >
    <div style="display: flex; height: calc(80vh - 120px)">
      <div style="width: 220px; flex-shrink: 0; overflow: auto; border-right: 1px solid var(--n-border-color); padding: 8px">
        <NTree
          :data="toTreeOptions(files)"
          :pattern=""
          block-line
          selectable
          @update:selected-keys="handleSelect"
        />
      </div>

      <div style="flex: 1; overflow: hidden; display: flex; flex-direction: column">
        <NScrollbar style="flex: 1">
          <div style="padding: 16px">
            <NSpin :show="loading">
              <div v-if="selectedFile">
                <NText depth="3" style="font-size: 12px; margin-bottom: 8px; display: block">
                  {{ selectedFile }}
                </NText>
                <NDivider style="margin: 8px 0" />
                <div v-if="isMarkdown" class="preview-markdown" v-html="renderedContent" />
                <div v-else class="preview-code" v-html="renderedContent" />
              </div>
              <div v-else style="text-align: center; color: #999; padding: 32px">
                {{ t("skills.preview.selectFile") }}
              </div>
            </NSpin>
          </div>
        </NScrollbar>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.preview-markdown {
  line-height: 1.7;
  font-size: 14px;
}
.preview-markdown :deep(h1), .preview-markdown :deep(h2), .preview-markdown :deep(h3) {
  margin-top: 16px;
  margin-bottom: 8px;
}
.preview-markdown :deep(code) {
  background: var(--n-color-embedded);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 13px;
}
.preview-markdown :deep(pre) {
  background: var(--n-color-embedded);
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
}
.preview-markdown :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 8px 0;
}
.preview-markdown :deep(th), .preview-markdown :deep(td) {
  border: 1px solid var(--n-border-color);
  padding: 6px 12px;
  text-align: left;
}
.preview-code :deep(pre) {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
}
.preview-code :deep(.hljs) {
  background: transparent;
}
</style>
```

- [ ] **Step 2: Verify component compiles**

Run: `npx vue-tsc --noEmit 2>&1 | grep -i "SkillPreviewModal" || echo "No errors for SkillPreviewModal"`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/plugins/skills/components/SkillPreviewModal.vue
git commit -m "feat(skills): add SkillPreviewModal with file tree and syntax highlighting"
```

---

## Task 10: Frontend — Integrate into SkillsMainView + i18n

**Files:**
- Modify: `src/plugins/skills/views/SkillsMainView.vue`
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`

- [ ] **Step 1: Update SkillsMainView.vue**

Replace the full file content:

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NSpace, NText, NButton, NTab, NTabs } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import ScopeTabs from "../components/ScopeTabs.vue";
import ProjectSelector from "../components/ProjectSelector.vue";
import SkillList from "../components/SkillList.vue";
import BatchActionBar from "../components/BatchActionBar.vue";
import SkillCompareTable from "../components/SkillCompareTable.vue";
import SkillDiffViewer from "../components/SkillDiffViewer.vue";
import SkillPreviewModal from "../components/SkillPreviewModal.vue";
import { GitCompareOutline } from "@vicons/ionicons5";

const { t } = useI18n();
const store = useSkillsStore();

const showDiff = ref(false);
const diffLocalPath = ref("");
const diffRemotePath = ref("");

const showPreview = ref(false);
const previewSkillPath = ref("");
const previewSkillName = ref("");

onMounted(() => {
  store.scanSkills(store.currentAgentId, store.currentScope);
});

function handleDiff(localPath: string, remotePath: string) {
  diffLocalPath.value = localPath;
  diffRemotePath.value = remotePath;
  showDiff.value = true;
}

function handlePreview(skillPath: string, skillName: string) {
  previewSkillPath.value = skillPath;
  previewSkillName.value = skillName;
  showPreview.value = true;
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace justify="space-between" align="center">
        <NText strong style="font-size: 18px">
          {{ t("skills.title") }}
        </NText>
        <NButton
          size="small"
          :type="store.comparisonMode ? 'primary' : 'default'"
          @click="store.toggleComparisonMode()"
        >
          <template #icon><GitCompareOutline /></template>
          {{ store.comparisonMode ? t("skills.compare.modeOn") : t("skills.compare.modeOff") }}
        </NButton>
      </NSpace>

      <AgentSelect />

      <ScopeTabs />

      <ProjectSelector v-if="store.currentScope === 'project'" />

      <SkillCompareTable
        v-if="store.comparisonMode"
        @diff="handleDiff"
        @preview="handlePreview"
      />

      <SkillList v-else />
    </NSpace>

    <BatchActionBar v-if="!store.comparisonMode" />

    <SkillDiffViewer
      :show="showDiff"
      :local-path="diffLocalPath"
      :remote-path="diffRemotePath"
      @close="showDiff = false"
    />

    <SkillPreviewModal
      :show="showPreview"
      :skill-path="previewSkillPath"
      :skill-name="previewSkillName"
      @close="showPreview = false"
    />
  </div>
</template>
```

- [ ] **Step 2: Update zh-CN.json**

Add these keys inside the `"skills"` object, after the existing `"confirm"` section:

```json
    "compare": {
      "status": "状态",
      "name": "名称",
      "sourceRepo": "来源仓库",
      "localVersion": "本地版本",
      "remoteVersion": "远端版本",
      "description": "描述",
      "actions": "操作",
      "statusSame": "一致",
      "statusOutdated": "可更新",
      "statusLocalOnly": "仅本地",
      "statusRemoteOnly": "仅远端",
      "viewDiff": "差异",
      "reinstall": "重新安装",
      "reinstallTip": "本地与远端一致，重新安装远端版本",
      "preview": "预览",
      "batchInstall": "批量安装",
      "batchUpdate": "批量更新",
      "modeOn": "对比模式",
      "modeOff": "开启对比"
    },
    "diff": {
      "title": "差异详情：{name}",
      "file": "文件",
      "status": "状态",
      "added": "新增",
      "removed": "删除",
      "modified": "修改",
      "unchanged": "未变",
      "loading": "加载中...",
      "selectFile": "点击左侧文件查看差异"
    },
    "preview": {
      "title": "预览：{name}",
      "selectFile": "点击左侧文件查看内容",
      "loadError": "加载失败"
    }
```

- [ ] **Step 3: Update en-US.json**

Add the same keys in English:

```json
    "compare": {
      "status": "Status",
      "name": "Name",
      "sourceRepo": "Source Repo",
      "localVersion": "Local Ver",
      "remoteVersion": "Remote Ver",
      "description": "Description",
      "actions": "Actions",
      "statusSame": "Same",
      "statusOutdated": "Outdated",
      "statusLocalOnly": "Local Only",
      "statusRemoteOnly": "Remote Only",
      "viewDiff": "Diff",
      "reinstall": "Reinstall",
      "reinstallTip": "Local matches remote. Reinstall from remote.",
      "preview": "Preview",
      "batchInstall": "Batch Install",
      "batchUpdate": "Batch Update",
      "modeOn": "Compare Mode",
      "modeOff": "Compare"
    },
    "diff": {
      "title": "Diff: {name}",
      "file": "File",
      "status": "Status",
      "added": "Added",
      "removed": "Removed",
      "modified": "Modified",
      "unchanged": "Unchanged",
      "loading": "Loading...",
      "selectFile": "Click a file on the left to view diff"
    },
    "preview": {
      "title": "Preview: {name}",
      "selectFile": "Click a file on the left to view content",
      "loadError": "Failed to load"
    }
```

- [ ] **Step 4: Verify full project compiles**

Run: `npx vue-tsc --noEmit 2>&1 | tail -5`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/plugins/skills/views/SkillsMainView.vue src/plugins/skills/i18n/zh-CN.json src/plugins/skills/i18n/en-US.json
git commit -m "feat(skills): integrate comparison mode into SkillsMainView with i18n"
```

---

## Task 11: End-to-End Verification

**Files:** None (testing only)

- [ ] **Step 1: Run Rust build**

Run: `cd src-tauri && cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 2: Run frontend build**

Run: `npm run build 2>&1 | tail -5`
Expected: Build success

- [ ] **Step 3: Start dev server and verify UI**

Run: `npm run tauri dev`

Verify:
1. Navigate to Skill Management page
2. Click "Compare" button → comparison mode activates
3. Comparison table shows with status columns
4. Click "Diff" on an outdated skill → diff modal opens
5. Click a file in diff → line-level diff renders
6. Click "Preview" → preview modal opens with file tree
7. Click a file in tree → content renders with highlighting
8. Toggle back to normal mode → SkillList returns

- [ ] **Step 4: Fix any issues found during testing**

- [ ] **Step 5: Final commit with any fixes**

```bash
git add -A
git commit -m "fix(skills): address issues found during Phase 2 testing"
```

---

## Self-Review Checklist

### Spec Coverage

| Requirement | Task |
|-------------|------|
| SkillCompareTable (local vs remote comparison) | Tasks 7, 10 |
| SkillDiffViewer (file-level + line-level diff) | Tasks 8, 10 |
| SkillPreviewModal (file tree + content preview) | Tasks 9, 10 |
| Backend comparison logic (hash-based) | Tasks 2, 3 |
| Backend diff logic (file-level) | Tasks 2, 3 |
| Batch install/update operations | Task 7 |
| Status badges (Same/Outdated/LocalOnly/RemoteOnly) | Tasks 1, 7 |
| i18n support (zh-CN + en-US) | Task 10 |
| Integration with existing SkillsMainView | Task 10 |

### Placeholder Scan

No TBD, TODO, "implement later", or placeholder steps found.

### Type Consistency

- `SkillComparison` defined in Task 1 (Rust) and Task 5 (TS) — fields match via serde `rename_all = "camelCase"`
- `FileDiffEntry` defined in Task 1 (Rust) and Task 5 (TS) — fields match
- `SkillDiffResult` defined in Task 1 (Rust) and Task 5 (TS) — fields match
- `DiffFileContent` defined in Task 1 (Rust) and Task 5 (TS) — fields match
- Store methods in Task 6 reference types from Task 5 — consistent
- Components in Tasks 7-9 reference store methods from Task 6 — consistent
