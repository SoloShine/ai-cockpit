# Skill 插件功能迁移 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 field-skill-manage (SPM Manager v1.3.0) 中尚未迁移的功能完整迁移到 ai-cockpit 的 Skill 管理插件中。

**Architecture:** 按 7 个独立 Task 拆分，每个 Task 包含前端组件、Rust 后端命令/服务、i18n、store 集成。Task 之间依赖关系弱，可顺序执行。

**Tech Stack:** Vue 3 + Naive UI + Pinia (前端), Rust + Tauri 2 IPC (后端), vue-i18n (国际化)

**参考项目:** `D:\Project\field-skill-manage`

---

## 文件结构总览

### 新建文件
```
src/plugins/skills/
├── components/
│   ├── MigrateDialog.vue          # Task 1
│   ├── OperationHistoryPanel.vue  # Task 2
│   └── SkillbasePanel.vue         # Task 3
├── views/
│   └── GuideView.vue              # Task 5

src-tauri/src/
├── commands/
│   └── history.rs                 # Task 2 (新建)
├── services/
│   └── history_service.rs         # Task 2 (新建)
```

### 修改文件
```
src/plugins/skills/
├── store.ts                       # Task 1, 2, 3 — 增加 migration/history/skillbase 方法
├── types.ts                       # Task 1, 2, 3 — 增加相关类型
├── i18n/zh-CN.json                # 所有 Task — 增加 i18n key
├── i18n/en-US.json                # 所有 Task — 增加 i18n key
├── views/SkillsMainView.vue       # Task 2 — 挂载历史面板
├── views/ProjectDetailView.vue    # Task 2 — 挂载历史面板
├── index.ts                       # Task 5 — 注册 Guide 路由

src-tauri/src/
├── commands/mod.rs                # Task 2 — 注册 history 模块
├── lib.rs                         # Task 2 — 注册 history 命令
├── services/git_service.rs        # Task 4 — 增加三级恢复
```

---

## Task 1: 跨 Agent 迁移向导 (MigrateDialog)

**Files:**
- Create: `src/plugins/skills/components/MigrateDialog.vue`
- Modify: `src/plugins/skills/store.ts`
- Modify: `src/plugins/skills/types.ts`
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`
- Reference: `D:\Project\field-skill-manage\src\components\common\MigrateDialog.vue`

**描述：** 3 步向导组件，允许用户将技能从一个 Agent 迁移到另一个 Agent。步骤为：选择源 Agent → 选择要迁移的 Skills → 解决冲突并执行迁移。

### 类型定义 (types.ts)

- [ ] **Step 1: 添加迁移相关类型**

在 `src/plugins/skills/types.ts` 中添加：

```typescript
/** 迁移时单个 skill 的扫描结果 */
export interface MigrateSkillItem {
  name: string
  sourcePath: string
  targetPath: string
  status: 'newTarget' | 'sameContent' | 'differentVersion' | 'contentDiffers'
  sourceHash?: string
  targetHash?: string
}

/** 迁移冲突解决策略 */
export type ConflictResolution = 'skip' | 'overwrite'

/** 迁移请求中的单个 skill */
export interface MigrateSkillRequest {
  name: string
  sourcePath: string
  targetPath: string
  resolution: ConflictResolution
}
```

### Store 方法 (store.ts)

- [ ] **Step 2: 在 store.ts 中添加迁移相关方法**

在 `src/plugins/skills/store.ts` 的 `useSkillsStore` 中添加以下 action：

```typescript
// === Migration State ===
showMigrateDialog: false,
migrateSourceAgentId: null as string | null,
migrateSkills: [] as MigrateSkillItem[],
migrateConflictResolutions: {} as Record<string, ConflictResolution>,

// === Migration Actions ===
async scanAgentSkills(agentId: string): Promise<MigrateSkillItem[]> {
  const settingsStore = useSettingsStore()
  const agentConfig = settingsStore.agents.find(a => a.id === agentId)
  if (!agentConfig) return []

  const home = await getHome()
  const sourcePath = agentConfig.globalPath.replace('~', home) + '/skills'
  const currentAgentConfig = this.getCurrentAgentConfig()
  if (!currentAgentConfig) return []
  const targetPath = currentAgentConfig.globalPath.replace('~', home) + '/skills'

  const { invoke } = await import('@tauri-apps/api/core')
  const skills = await invoke<MigrateSkillItem[]>('scan_migrate_skills', {
    sourcePath,
    targetPath,
  })
  this.migrateSkills = skills
  return skills
},

async migrateSkillsAction(requests: MigrateSkillRequest[]): Promise<OperationResult[]> {
  const { invoke } = await import('@tauri-apps/api/core')
  const results = await invoke<OperationResult[]>('migrate_skills', { requests })
  await this.loadComparisons()
  return results
},
```

### Rust 后端

- [ ] **Step 3: 在 skills.rs 中添加迁移命令**

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateSkillItem {
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: String,
    pub source_hash: Option<String>,
    pub target_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateSkillRequest {
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub resolution: String,
}

#[tauri::command]
pub async fn scan_migrate_skills(
    source_path: String,
    target_path: String,
) -> Result<Vec<MigrateSkillItem>, String> {
    skills_service::scan_migrate_skills(&source_path, &target_path)
}

#[tauri::command]
pub async fn migrate_skills(
    requests: Vec<MigrateSkillRequest>,
) -> Result<Vec<OperationResult>, String> {
    skills_service::migrate_skills(requests)
}
```

- [ ] **Step 4: 在 skills_service.rs 中添加迁移逻辑**

```rust
pub fn scan_migrate_skills(source_path: &str, target_path: &str) -> Result<Vec<serde_json::Value>, String> {
    // 扫描 source_path 下的所有 skills
    // 对每个 skill，检查 target_path 下是否存在同名 skill
    // 比较 hash 确定状态：newTarget / sameContent / differentVersion / contentDiffers
    // 返回 MigrateSkillItem 列表
}

pub fn migrate_skills(requests: Vec<serde_json::Value>) -> Result<Vec<serde_json::Value>, String> {
    // 对每个请求：
    //   resolution=skip → 跳过
    //   resolution=overwrite → 复制 source 到 target（覆盖）
    // 记录操作历史
    // 返回结果列表
}
```

### 前端组件

- [ ] **Step 5: 创建 MigrateDialog.vue**

参考 `D:\Project\field-skill-manage\src\components\common\MigrateDialog.vue` 创建完整的 3 步向导：

1. **Step 1 - 选择源 Agent**：下拉选择源 Agent，点击"扫描"
2. **Step 2 - 选择 Skills**：checkbox 列表显示可迁移 skills，每行显示状态徽章（NewTarget=success, SameContent=default, DifferentVersion=warning, ContentDiffers=error）
3. **Step 3 - 解决冲突**：对 status=contentDiffers 的 skill 显示冲突解决选项（skip/overwrite），可点击查看 Diff

关键 UI 元素：
- `NModal` 外壳，宽度 700px
- `NSteps` 步骤指示器（currentStep ref）
- Step 2 中用 `NCheckbox` + `NTag` 状态徽章
- Step 3 中对冲突 skill 用 `NRadioGroup`（skip/overwrite）
- 底部 `NSpace` 放上一步/下一步/取消按钮
- 集成 `SkillDiffViewer` 用于冲突详情查看

### i18n

- [ ] **Step 6: 添加迁移相关 i18n key**

zh-CN.json 在 `skills` 下添加：
```json
"migrate": {
  "title": "迁移技能",
  "step1": "选择源 Agent",
  "step2": "选择技能",
  "step3": "解决冲突",
  "sourceAgent": "源 Agent",
  "selectSource": "请选择要从中迁移的 Agent",
  "scan": "扫描",
  "scanning": "扫描中...",
  "noSkills": "源 Agent 中没有技能",
  "selectAll": "全选",
  "deselectAll": "取消全选",
  "selected": "已选 {count} 个",
  "conflict": "冲突",
  "conflictDesc": "以下技能在目标 Agent 中已存在且内容不同",
  "skip": "跳过",
  "overwrite": "覆盖",
  "migrating": "迁移中...",
  "success": "迁移完成：{success} 成功，{fail} 失败",
  "statusNew": "新增",
  "statusSame": "内容相同",
  "statusDiff": "版本不同",
  "statusConflict": "内容冲突",
  "next": "下一步",
  "prev": "上一步",
  "confirm": "开始迁移"
}
```

en-US.json 对应翻译。

- [ ] **Step 7: 在 SkillsMainView.vue 中添加迁移按钮和对话框**

在 SkillsMainView 的操作栏中添加"迁移"按钮，点击后 `store.showMigrateDialog = true`，并挂载 `MigrateDialog` 组件。

- [ ] **Step 8: 在 lib.rs 中注册新命令**

将 `scan_migrate_skills` 和 `migrate_skills` 注册到 `invoke_handler`。

- [ ] **Step 9: Commit**

```bash
git add src/plugins/skills/components/MigrateDialog.vue \
        src/plugins/skills/store.ts \
        src/plugins/skills/types.ts \
        src-tauri/src/commands/skills.rs \
        src-tauri/src/services/skills_service.rs \
        src-tauri/src/lib.rs \
        src/plugins/skills/i18n/zh-CN.json \
        src/plugins/skills/i18n/en-US.json \
        src/plugins/skills/views/SkillsMainView.vue
git commit -m "feat(skills): add cross-agent migration wizard"
```

---

## Task 2: 操作历史与回滚 (OperationHistoryPanel)

**Files:**
- Create: `src-tauri/src/commands/history.rs`
- Create: `src-tauri/src/services/history_service.rs`
- Create: `src/plugins/skills/components/OperationHistoryPanel.vue`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src/plugins/skills/store.ts`
- Modify: `src/plugins/skills/types.ts`
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`
- Modify: `src/plugins/skills/views/SkillsMainView.vue`
- Modify: `src/plugins/skills/views/ProjectDetailView.vue`
- Reference: `D:\Project\field-skill-manage\src-tauri\src\services\history_service.rs`
- Reference: `D:\Project\field-skill-manage\src\components\common\OperationHistoryPanel.vue`

**描述：** 记录所有 install/update/uninstall 操作，支持查看历史和回滚。历史数据持久化到 JSON 文件，最多保留 200 条记录。

### Rust 后端

- [ ] **Step 1: 创建 history_service.rs**

参考旧项目 `D:\Project\field-skill-manage\src-tauri\src\services\history_service.rs`，核心结构：

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub id: String,           // UUID
    pub operation_type: String, // install, update, uninstall
    pub skill_name: String,
    pub target_path: String,
    pub source_path: Option<String>,
    pub timestamp: String,    // ISO 8601
    pub version_before: Option<String>,
    pub version_after: Option<String>,
    pub can_rollback: bool,
    pub rolled_back: bool,
}

const MAX_RECORDS: usize = 200;

fn history_file_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Cannot determine data directory")?;
    Ok(data_dir.join("ai-cockpit").join("history.json"))
}

pub fn load_history() -> Result<Vec<OperationRecord>, String> {
    let path = history_file_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read history: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse history: {}", e))
}

pub fn save_history(records: &[OperationRecord]) -> Result<(), String> {
    let path = history_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create history dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write history: {}", e))
}

pub fn record_operation(
    operation_type: &str,
    skill_name: &str,
    target_path: &str,
    source_path: Option<&str>,
    version_before: Option<&str>,
    version_after: Option<&str>,
) -> Result<OperationRecord, String> {
    let mut records = load_history()?;
    let record = OperationRecord {
        id: uuid::Uuid::new_v4().to_string(),
        operation_type: operation_type.to_string(),
        skill_name: skill_name.to_string(),
        target_path: target_path.to_string(),
        source_path: source_path.map(String::from),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version_before: version_before.map(String::from),
        version_after: version_after.map(String::from),
        can_rollback: true,
        rolled_back: false,
    };
    records.insert(0, record.clone());
    records.truncate(MAX_RECORDS);
    save_history(&records)?;
    Ok(record)
}

pub fn rollback_operation(id: &str) -> Result<String, String> {
    let mut records = load_history()?;
    let record = records.iter_mut()
        .find(|r| r.id == id)
        .ok_or("Operation not found")?;

    if record.rolled_back {
        return Err("Operation already rolled back".to_string());
    }
    if !record.can_rollback {
        return Err("Operation cannot be rolled back".to_string());
    }

    match record.operation_type.as_str() {
        "install" => {
            // 卸载已安装的 skill
            let target = PathBuf::from(&record.target_path);
            if target.exists() {
                if target.is_dir() {
                    fs::remove_dir_all(&target)
                } else {
                    fs::remove_file(&target)
                }.map_err(|e| format!("Rollback install failed: {}", e))?;
            }
        }
        "uninstall" => {
            // 从 source 重新安装
            if let Some(source) = &record.source_path {
                let source_path = PathBuf::from(source);
                let target_path = PathBuf::from(&record.target_path);
                if source_path.is_dir() {
                    copy_dir_recursive(&source_path, &target_path)?;
                } else if source_path.exists() {
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("{}", e))?;
                    }
                    fs::copy(&source_path, &target_path)
                        .map_err(|e| format!("{}", e))?;
                }
            }
        }
        "update" => {
            // best-effort: 如果有 source，重新复制
            if let Some(source) = &record.source_path {
                let source_path = PathBuf::from(source);
                let target_path = PathBuf::from(&record.target_path);
                if source_path.is_dir() {
                    if target_path.exists() {
                        fs::remove_dir_all(&target_path)
                            .map_err(|e| format!("{}", e))?;
                    }
                    copy_dir_recursive(&source_path, &target_path)?;
                }
            }
        }
        _ => return Err(format!("Unknown operation type: {}", record.operation_type)),
    }

    record.rolled_back = true;
    save_history(&records)?;
    Ok(format!("Rolled back {} for {}", record.operation_type, record.skill_name))
}

pub fn clear_history() -> Result<(), String> {
    let path = history_file_path()?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("{}", e))?;
    }
    Ok(())
}

// 辅助：递归复制目录 (复用 skills_service 中已有的逻辑或抽取公共函数)
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    // 同 skills_service 中的 copy_dir_all
}
```

注意：需要在 Cargo.toml 中添加 `uuid` 和 `chrono` 依赖（如果还没有的话）。

- [ ] **Step 2: 创建 commands/history.rs**

```rust
use crate::services::history_service;

#[tauri::command]
pub async fn get_operation_history(
    limit: Option<usize>,
) -> Result<Vec<serde_json::Value>, String> {
    let records = history_service::load_history()?;
    let limited = match limit {
        Some(n) => records.into_iter().take(n).collect(),
        None => records,
    };
    Ok(limited.iter().map(|r| serde_json::to_value(r).unwrap()).collect())
}

#[tauri::command]
pub async fn rollback_operation(id: String) -> Result<String, String> {
    history_service::rollback_operation(&id)
}

#[tauri::command]
pub async fn clear_history() -> Result<(), String> {
    history_service::clear_history()
}
```

- [ ] **Step 3: 注册 history 模块**

在 `src-tauri/src/commands/mod.rs` 中添加 `pub mod history;`
在 `src-tauri/src/services/mod.rs` 中添加 `pub mod history_service;`
在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册：
```rust
.invoke_handler(tauri::generate_handler![
    // ...existing commands...
    commands::history::get_operation_history,
    commands::history::rollback_operation,
    commands::history::clear_history,
])
```

- [ ] **Step 4: 在 Cargo.toml 中添加依赖**

检查并添加（如不存在）：
```toml
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 前端类型 (types.ts)

- [ ] **Step 5: 添加历史相关类型**

```typescript
export interface OperationRecord {
  id: string
  operationType: 'install' | 'update' | 'uninstall'
  skillName: string
  targetPath: string
  sourcePath?: string
  timestamp: string
  versionBefore?: string
  versionAfter?: string
  canRollback: boolean
  rolledBack: boolean
}
```

### Store 集成 (store.ts)

- [ ] **Step 6: 在 store 中添加历史方法**

```typescript
// State
operationHistory: [] as OperationRecord[],

// Actions
async getOperationHistory(limit?: number) {
  const { invoke } = await import('@tauri-apps/api/core')
  this.operationHistory = await invoke<OperationRecord[]>('get_operation_history', { limit })
},

async rollbackOperation(id: string) {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke<string>('rollback_operation', { id })
  await this.getOperationHistory()
  await this.loadComparisons()
},

async clearHistory() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('clear_history')
  this.operationHistory = []
},
```

同时在 `installSkill`、`updateSkill`、`uninstallSkill` 方法成功后调用 `record_operation` 后端命令记录操作。

### 前端组件

- [ ] **Step 7: 创建 OperationHistoryPanel.vue**

参考旧项目 `D:\Project\field-skill-manage\src\components\common\OperationHistoryPanel.vue`：

- `NDrawer` 或面板容器，placement="right"，width 400px
- 列表项显示：操作类型 NTag（Install=success, Update=warning, Uninstall=error）、skill 名称、目标路径、时间
- 每条记录有"回滚"按钮（`canRollback && !rolledBack` 时可用）
- 顶部"清空历史"按钮 + `NPopconfirm` 确认
- `NEmpty` 空状态
- `NScrollbar` 可滚动
- `NSpin` 加载状态

### 集成

- [ ] **Step 8: 在 SkillsMainView.vue 和 ProjectDetailView.vue 中添加历史按钮**

在操作栏添加"历史"按钮，点击显示 OperationHistoryPanel。挂载组件。

### i18n

- [ ] **Step 9: 添加历史相关 i18n key**

zh-CN.json：
```json
"history": {
  "title": "操作历史",
  "empty": "暂无操作记录",
  "install": "安装",
  "update": "更新",
  "uninstall": "卸载",
  "rollback": "回滚",
  "rollbackConfirm": "确定要回滚此操作吗？",
  "rollbackSuccess": "回滚成功",
  "rollbackFailed": "回滚失败",
  "clear": "清空历史",
  "clearConfirm": "确定要清空所有操作记录吗？此操作不可撤销。",
  "rolledBack": "已回滚",
  "target": "目标路径",
  "time": "时间"
}
```

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat(skills): add operation history with rollback support"
```

---

## Task 3: Skillbase 依赖管理 (SkillbasePanel)

**Files:**
- Create: `src/plugins/skills/components/SkillbasePanel.vue`
- Modify: `src/plugins/skills/store.ts`
- Modify: `src/plugins/skills/types.ts`
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`
- Reference: `D:\Project\field-skill-manage\src\components\common\SkillbasePanel.vue`

**描述：** 解析 skillbase.json 依赖声明，检查本地安装状态，提供同步和重新生成功能。

### 类型定义

- [ ] **Step 1: 添加 skillbase 相关类型**

```typescript
export interface SkillbaseDependency {
  name: string
  version?: string
  status: 'satisfied' | 'missing' | 'mismatch' | 'outdated'
  localVersion?: string
  requiredVersion?: string
}

export interface SkillbaseResolution {
  manifestName: string
  skillDir: string
  dependencies: SkillbaseDependency[]
  totalSkills: number
  satisfiedCount: number
  missingCount: number
}
```

### Rust 后端

- [ ] **Step 2: 在 skills.rs / skills_service.rs 中添加 skillbase 命令**

```rust
#[tauri::command]
pub async fn resolve_skillbase(
    skill_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<serde_json::Value, String> {
    skills_service::resolve_skillbase(&skill_dir, &repos)
}

#[tauri::command]
pub async fn sync_skillbase(
    skill_dir: String,
    repos: Vec<RepoConfig>,
) -> Result<Vec<serde_json::Value>, String> {
    skills_service::sync_skillbase(&skill_dir, &repos)
}

#[tauri::command]
pub async fn generate_skillbase(
    skill_dir: String,
) -> Result<(), String> {
    skills_service::generate_skillbase(&skill_dir)
}
```

服务层逻辑：
- `resolve_skillbase`: 读取 skillbase.json，解析依赖列表，检查本地是否安装，比对版本
- `sync_skillbase`: 对 missing/mismatch 的依赖从远端仓库安装
- `generate_skillbase`: 扫描 skill 目录下所有已安装 skill，自动生成 skillbase.json

### 前端组件

- [ ] **Step 3: 创建 SkillbasePanel.vue**

参考旧项目 `D:\Project\field-skill-manage\src\components\common\SkillbasePanel.vue`：

- 面板标题：manifest 名称 + 统计 (satisfied/missing)
- 依赖列表：每行显示 skill 名称、状态色点、版本信息
- 状态 Tag：satisfied=success, missing=error, mismatch=warning, outdated=warning
- 操作按钮："同步依赖"和"重新生成"
- `NEmpty` 空状态

### Store 集成

- [ ] **Step 4: 在 store 中添加 skillbase 方法**

```typescript
skillbase: null as SkillbaseResolution | null,

async loadSkillbase(skillDir: string) {
  const { invoke } = await import('@tauri-apps/api/core')
  this.skillbase = await invoke<SkillbaseResolution>('resolve_skillbase', {
    skillDir,
    repos: settingsStore.repos,
  })
},

async syncSkillbase(skillDir: string) {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('sync_skillbase', { skillDir, repos: settingsStore.repos })
  await this.loadSkillbase(skillDir)
  await this.loadComparisons()
},

async generateSkillbase(skillDir: string) {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('generate_skillbase', { skillDir })
  await this.loadSkillbase(skillDir)
},
```

### i18n

- [ ] **Step 5: 添加 skillbase i18n key**

zh-CN.json：
```json
"skillbase": {
  "title": "Skillbase 依赖",
  "manifest": "清单",
  "stats": "{total} 个依赖，{satisfied} 已满足，{missing} 缺失",
  "satisfied": "已满足",
  "missing": "缺失",
  "mismatch": "不匹配",
  "outdated": "过时",
  "sync": "同步依赖",
  "syncing": "同步中...",
  "generate": "重新生成",
  "generating": "生成中...",
  "noDependencies": "没有依赖声明",
  "syncSuccess": "依赖同步完成",
  "generateSuccess": "skillbase.json 已重新生成"
}
```

- [ ] **Step 6: 集成到 SkillsMainView / ProjectDetailView**

在 skill 列表上方或操作栏中添加"依赖管理"按钮，点击展开 SkillbasePanel。

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(skills): add skillbase dependency management"
```

---

## Task 4: Git 同步健壮性增强

**Files:**
- Modify: `src-tauri/src/services/git_service.rs` (如果新项目有此文件) 或 `src-tauri/src/commands/skills.rs` 中的 git 相关逻辑
- Reference: `D:\Project\field-skill-manage\src-tauri\src\services\git_service.rs`

**描述：** 将旧项目 git_service.rs 中的三级恢复机制迁移到新项目。当前新项目的 git 同步缺少损坏检测和自动恢复。

- [ ] **Step 1: 检查新项目当前 git 同步实现**

检查新项目中 git 同步逻辑在哪个文件，了解当前实现状态。

- [ ] **Step 2: 实现三级恢复机制**

参考旧项目 `git_service.rs` 中的 `sync_repo` 函数：

```
策略 1: git pull --ff-only（常规更新）
    ↓ 失败
策略 2: git fetch + git reset --hard origin/main（强制同步）
    ↓ 失败
策略 3: 删除目录 + git clone --depth 1（完全重建）
```

关键实现点：
- `is_valid_git_repo()`: 检查 .git 目录是否有效
- `sync_repo()`: 主入口，封装三级策略
- `get_latest_commit_time()`: 获取最后提交时间用于显示
- 错误处理：每级失败后自动降级，不中断用户流程

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(skills): add robust git sync with 3-level recovery"
```

---

## Task 5: 帮助/指南页面 (GuideView)

**Files:**
- Create: `src/plugins/skills/views/GuideView.vue`
- Modify: `src/plugins/skills/index.ts` (添加路由)
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`
- Reference: `D:\Project\field-skill-manage\src\views\GuideView.vue`

**描述：** 交互式 Skill 开发指南页面，包含目录结构示例、Frontmatter 字段说明、Trigger 配置、Security 权限、依赖管理、Body 编写指南等。

- [ ] **Step 1: 创建 GuideView.vue**

参考旧项目 `D:\Project\field-skill-manage\src\views\GuideView.vue`，核心结构：

- 双栏布局：左侧 TOC 导航 + 右侧内容区
- 内容章节（从 i18n 数据或组件内 data）：
  1. 目录结构
  2. Frontmatter 必填/选填字段
  3. Trigger 配置（description, tags, file_patterns, priority）
  4. Security 权限声明
  5. Dependencies 依赖管理
  6. skillbase.json 规范
  7. Body 编写指南（XML 标签使用）
  8. 完整示例
  9. 验证与发布流程
- 代码块使用 `NCode` 组件 + 语法高亮
- IntersectionObserver 追踪当前活跃章节，TOC 高亮
- 代码块支持一键复制

- [ ] **Step 2: 注册路由**

在 `src/plugins/skills/index.ts` 的 routes 数组中添加：
```typescript
{
  path: '/skills/guide',
  name: 'skills-guide',
  component: () => import('./views/GuideView.vue'),
  meta: { pluginId: 'skills' },
}
```

在 navItems 中添加"开发指南"导航项。

- [ ] **Step 3: i18n key**

zh-CN.json：
```json
"guide": {
  "title": "Skill 开发指南",
  "toc": "目录",
  "directory": "目录结构",
  "frontmatter": "Frontmatter",
  "trigger": "触发器配置",
  "security": "安全权限",
  "dependencies": "依赖管理",
  "skillbase": "skillbase.json",
  "body": "Body 编写",
  "example": "完整示例",
  "publishing": "验证与发布",
  "copyCode": "复制代码",
  "copied": "已复制"
}
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(skills): add interactive skill development guide"
```

---

## Task 6: 配置导入导出

**Files:**
- Modify: `src/plugins/skills/views/SkillsMainView.vue`（或设置页面）
- Modify: `src/plugins/skills/store.ts`
- Modify: `src/plugins/skills/i18n/zh-CN.json`
- Modify: `src/plugins/skills/i18n/en-US.json`

**描述：** 支持导出当前仓库配置、Agent 配置为 JSON 文件，以及从 JSON 文件导入恢复。

- [ ] **Step 1: 在 store 中添加 export/import 方法**

```typescript
async exportConfig(): Promise<string> {
  const config = {
    projectPaths: this.projectPaths,
    // agent 和 repo 配置从 settingsStore 获取
  }
  return JSON.stringify(config, null, 2)
},

async importConfig(jsonStr: string): Promise<void> {
  const config = JSON.parse(jsonStr)
  if (config.projectPaths) {
    for (const p of config.projectPaths) {
      this.addProject(p)
    }
  }
},
```

Settings store 中也需要对应的 repo 配置导入导出。

- [ ] **Step 2: 在 UI 中添加导入导出按钮**

在 Skill 管理页面或设置页中添加"导出配置"和"导入配置"按钮。
使用 `NButton` + Tauri 文件对话框（`dialog.save()` / `dialog.open()`）。

- [ ] **Step 3: i18n**

```json
"config": {
  "export": "导出配置",
  "import": "导入配置",
  "exportSuccess": "配置已导出",
  "importSuccess": "配置已导入",
  "importFailed": "导入失败：格式错误"
}
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(skills): add config export/import"
```

---

## Task 7: 集成验证与清理

**Files:**
- All modified files

**描述：** 所有功能迁移完成后，进行端到端验证。

- [ ] **Step 1: TypeScript 编译检查**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 2: Rust 编译检查**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: 启动开发服务器验证**

```bash
npm run tauri dev
```

逐个验证：
1. 迁移向导：选择源 Agent → 扫描 → 选择 Skills → 解决冲突 → 迁移
2. 操作历史：安装/更新/卸载 skill 后查看历史 → 回滚
3. Skillbase：查看依赖状态 → 同步缺失依赖 → 重新生成
4. Git 同步：损坏 repo 后同步，验证自动恢复
5. 指南页面：目录导航、代码复制正常工作
6. 配置导入导出：导出 JSON → 清空 → 导入恢复

- [ ] **Step 4: 清理未使用的代码**

移除旧项目中已废弃但新项目中误引入的代码，确保无 dead code 警告。

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore(skills): verify and clean up migrated features"
```

---

## 执行优先级建议

| Task | 优先级 | 依赖 | 预计工作量 |
|------|--------|------|-----------|
| Task 2 (操作历史) | P0 | 无 | 中 |
| Task 4 (Git 健壮性) | P0 | 无 | 中 |
| Task 1 (迁移向导) | P1 | 无 | 高 |
| Task 3 (Skillbase) | P1 | Task 2 | 中 |
| Task 5 (指南页面) | P2 | 无 | 中 |
| Task 6 (配置导入导出) | P2 | 无 | 低 |
| Task 7 (验证清理) | P0 | 所有 | 低 |

建议执行顺序：**Task 2 → Task 4 → Task 1 → Task 3 → Task 5 → Task 6 → Task 7**
