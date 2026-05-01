# Skill 管理插件迁移设计

> 从 `field-skill-manage` 迁移 Skill 管理功能到 `ai-cockpit` 作为标准插件

## 概述

将 `field-skill-manage`（SPM Manager）的核心 Skill 管理功能迁移到 ai-cockpit 的插件化架构中。采用**核心功能优先**策略，分 3 批交付，重写优化 Rust 后端，重新设计前端 UI。

### 迁移范围

| 批次 | 功能 | Rust 命令 | 前端页面 | 状态 |
|------|------|-----------|----------|------|
| Batch 1 | Skill 扫描 + 展示 | 6 | 主列表页 + Agent Tab | 核心 |
| Batch 2 | 安装/更新/卸载 | 5 | 操作 UI + 确认流程 | 核心 |
| Batch 3 | 项目管理 + 预览 + 版本对比 + 迁移 | 6 | 项目视图 + 预览 + 对比 | 后期 |

### 关键决策

1. **配置模型**：复用 Settings 插件的 `AgentConfig`（10 个 Agent），Skill 插件不自带配置
2. **Rust 策略**：重写优化，更模块化、无状态设计
3. **UI 策略**：重新设计，卡片式布局 + Agent Tab + 全局/项目 Tab
4. **迁移方案**：按功能域分批交付（方案 A）

## 插件架构

### 目录结构

```
src/plugins/skills/
├── index.ts                    # CockpitPlugin + PluginHooks 导出
├── types.ts                    # 插件内部类型定义
├── i18n/
│   ├── zh-CN.json
│   └── en-US.json
├── views/
│   ├── SkillsMainView.vue      # 主视图（范围 Tab + Agent Tab + Skill 列表）
│   └── SkillDetailView.vue     # Skill 详情/预览（右侧抽屉）
├── components/
│   ├── AgentTabs.vue           # Agent 切换标签（水平可滚动）
│   ├── ScopeTabs.vue           # 全局/项目范围切换
│   ├── SkillCard.vue           # 单个 Skill 卡片
│   ├── SkillList.vue           # Skill 列表容器（卡片网格）
│   ├── SkillActions.vue        # 安装/更新/卸载操作栏
│   ├── SkillPreview.vue        # Skill 文件预览（后期）
│   ├── BatchActionBar.vue      # 批量操作状态栏
│   └── EmptyState.vue          # 空状态/引导提示
├── store.ts                    # Pinia store
└── composables.ts              # 公共 API
```

### Rust 后端结构

```
src-tauri/src/
├── commands/
│   └── skills.rs               # Skill IPC 命令（无状态）
├── services/
│   └── skills_service.rs       # Skill 业务逻辑（重写优化）
└── models/
    └── skills.rs               # Skill 数据模型
```

## 数据模型

### Rust 模型（`models/skills.rs`）

```rust
use serde::{Deserialize, Serialize};

/// Skill 元信息（从 SKILL.md 或 skills.json 解析）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,     // skillbase 依赖（预留）
}

/// 单个 Skill 的完整信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub name: String,
    pub path: String,
    pub is_file: bool,
    pub has_skill_md: bool,
    pub meta: Option<SkillMeta>,
    pub file_count: usize,
    pub size_bytes: u64,
    // 版本对比预留字段
    pub content_hash: String,              // SHA256 内容哈希
    pub last_modified: Option<String>,     // 最后修改时间
    // 来源信息（跨 Agent 迁移预留）
    pub source_agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SkillScope {
    Global,
    Project,
}

/// 扫描结果
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub agent_id: String,
    pub scope: SkillScope,
    pub skills: Vec<SkillInfo>,
    pub total: usize,
}

/// 文件树条目
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<FileEntry>,
}

/// 版本对比状态
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DiffStatus {
    Same,
    Modified,
    Added,
    Removed,
}

/// 版本对比结果（预留）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiff {
    pub name: String,
    pub source_hash: String,
    pub target_hash: String,
    pub status: DiffStatus,
    pub changed_files: Vec<FileDiff>,
}

/// 文件 Diff（预留）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub diff_type: DiffStatus,
    pub content: Option<String>,
}

/// 操作结果
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub affected_paths: Vec<String>,
}

/// 批量操作类型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

/// 批量操作条目
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillOperation {
    pub operation_type: OperationType,
    pub source: String,
    pub target_path: String,
}

/// 项目概览（多 Agent 统计）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverview {
    pub project_path: String,
    pub project_name: String,
    pub agent_skills_count: HashMap<String, usize>,
}

/// 项目详情
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project_path: String,
    pub project_name: String,
    pub agents: Vec<AgentSkillInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkillInfo {
    pub agent_id: String,
    pub skills: Vec<SkillInfo>,
    pub total: usize,
}
```

### 前端类型（`types.ts`）

```typescript
export interface SkillMeta {
  name: string;
  description: string;
  version?: string;
  author?: string;
  tags: string[];
  dependencies: string[];
}

export interface SkillInfo {
  name: string;
  path: string;
  isFile: boolean;
  hasSkillMd: boolean;
  meta?: SkillMeta;
  fileCount: number;
  sizeBytes: number;
  contentHash: string;
  lastModified?: string;
  sourceAgentId?: string;
}

export type SkillScope = "global" | "project";

export interface ScanResult {
  agentId: string;
  scope: SkillScope;
  skills: SkillInfo[];
  total: number;
}

export interface FileEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  children: FileEntry[];
}

export type DiffStatus = "same" | "modified" | "added" | "removed";

export interface SkillDiff {
  name: string;
  sourceHash: string;
  targetHash: string;
  status: DiffStatus;
  changedFiles: FileDiff[];
}

export interface FileDiff {
  path: string;
  diffType: DiffStatus;
  content?: string;
}

export interface OperationResult {
  success: boolean;
  message: string;
  affectedPaths: string[];
}

export type OperationType = "install" | "update" | "uninstall";

export interface SkillOperation {
  operationType: OperationType;
  source: string;
  targetPath: string;
}

export interface ProjectOverview {
  projectPath: string;
  projectName: string;
  agentSkillsCount: Record<string, number>;
}

export interface ProjectDetail {
  projectPath: string;
  projectName: string;
  agents: AgentSkillInfo[];
}

export interface AgentSkillInfo {
  agentId: string;
  skills: SkillInfo[];
  total: number;
}
```

## IPC 命令

### Batch 1：扫描 + 展示

| 命令 | 签名 | 说明 |
|------|------|------|
| `scan_global_skills` | `(agent_id, global_path) → ScanResult` | 扫描指定 Agent 的全局 Skill 目录 |
| `scan_project_skills` | `(agent_id, project_path, project_dir) → ScanResult` | 扫描指定 Agent 的项目级 Skill |
| `get_skill_file_tree` | `(skill_path) → Vec<FileEntry>` | 获取 Skill 文件树 |
| `read_skill_file` | `(file_path) → String` | 读取 Skill 文件内容 |
| `calculate_skill_hash` | `(skill_path) → String` | 计算 Skill SHA256 哈希 |
| `get_projects_overview` | `(agent_ids) → Vec<ProjectOverview>` | 获取项目列表 |

### Batch 2：安装/更新/卸载

| 命令 | 签名 | 说明 |
|------|------|------|
| `install_skill` | `(source, target_path) → OperationResult` | 安装 Skill |
| `update_skill` | `(source, target_path) → OperationResult` | 更新 Skill |
| `uninstall_skill` | `(skill_path) → OperationResult` | 卸载 Skill |
| `batch_operate` | `(operations) → Vec<OperationResult>` | 批量操作 |
| `verify_skill_integrity` | `(skill_path, expected_hash) → bool` | 验证完整性 |

### Batch 3：项目管理 + 版本对比 + 迁移（后期）

| 命令 | 签名 | 说明 |
|------|------|------|
| `get_project_detail` | `(project_path, agent_ids) → ProjectDetail` | 项目详情 |
| `compare_skill_versions` | `(source, target) → SkillDiff` | 版本对比 |
| `get_diff_file_content` | `(source, target) → String` | 文件 Diff |
| `scan_agent_skills` | `(source_agent_path) → ScanResult` | 跨 Agent 扫描 |
| `migrate_skills` | `(source, target, resolution) → OperationResult` | 执行迁移 |
| `preview_remote_skill` | `(repo_url, skill_name) → SkillInfo` | 远程预览 |

**关键设计**：所有命令都是无状态的纯函数，路径参数从前端传入（来自 Settings 的 AgentConfig），不在 Rust 端维护配置状态。

## 前端 UI

### 布局结构

主视图采用顶部双 Tab + 卡片网格布局：

- **顶部 Tab 1**：范围切换 — `全局 Skills` / `项目 Skills`
- **顶部 Tab 2**：Agent 切换 — 水平可滚动标签，仅显示 enabled Agent
- **主内容区**：Skill 卡片网格（自适应列数）
- **右侧抽屉**：Skill 详情（文件树 + 预览）
- **底部状态栏**：选中计数 + 批量操作

### Skill 卡片内容

```
┌─────────────────────┐
│ ☐ Skill Name        │
│ 一行描述文本...       │
│                     │
│ 📁 3 files · 2.1KB  │
│ v1.0 · 2024-01-01   │
│                     │
│ [安装]      [▶ 详情] │
└─────────────────────┘
```

### 空状态处理

- Agent 路径未配置 → 显示引导提示，附带"前往设置"链接
- 路径存在但无 Skill → 显示空状态插图
- 扫描失败 → 显示错误信息 + 重试按钮

### 路由设计

```typescript
routes: [
  {
    path: "/skills",
    name: "skills",
    component: () => import("./views/SkillsMainView.vue"),
    meta: { pluginId: "skills" },
    children: [
      {
        path: "global",
        name: "skills-global",
        component: () => import("./views/SkillsMainView.vue"),
      },
      {
        path: "project",
        name: "skills-project",
        component: () => import("./views/SkillsMainView.vue"),
      },
    ],
  },
]
```

## Store 设计

```typescript
// src/plugins/skills/store.ts
export const useSkillsStore = defineStore("skills", () => {
  // 当前状态
  const currentAgentId = ref<string>("claude-code");
  const currentScope = ref<SkillScope>("global");
  const currentProjectPath = ref<string>("");

  // 缓存
  const globalSkills = ref<Map<string, ScanResult>>(new Map());
  const projectSkills = ref<Map<string, ScanResult>>(new Map());
  const selectedSkills = ref<Set<string>>(new Set());

  // UI 状态
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 核心方法
  async function scanSkills(agentId: string, scope: SkillScope) { ... }
  async function scanAllAgents(scope: SkillScope) { ... }
  async function installSkill(source: string, targetPath: string) { ... }
  async function updateSkill(source: string, targetPath: string) { ... }
  async function uninstallSkill(skillPath: string) { ... }
  async function batchOperate(operations: SkillOperation[]) { ... }

  // 辅助
  function getCurrentSkills(): SkillInfo[] { ... }
  function toggleSelect(skillName: string) { ... }
  function selectAll() { ... }
  function clearSelection() { ... }
});
```

## 跨插件通信

Skills 插件通过以下方式与 Settings 插件通信：

1. **直接 import Settings Store**（符合 CLAUDE.md 的 Pinia store 通信规范）
   ```typescript
   import { useSettingsStore } from "@/plugins/settings/store";
   ```

2. **读取 Agent 配置**：从 `useSettingsStore().agents` 获取路径信息

3. **响应配置变化**：watch `agents` 变化，路径被修改时自动重新扫描

4. **过滤 Agent**：Agent Tab 只显示 `enabled: true` 的 Agent

## i18n Key 格式

遵循 `skills.<section>.<key>` 格式：

```json
{
  "skills": {
    "title": "Skill 管理",
    "scope": {
      "global": "全局 Skills",
      "project": "项目 Skills"
    },
    "actions": {
      "install": "安装",
      "update": "更新",
      "uninstall": "卸载",
      "batch": "批量操作"
    },
    "status": {
      "empty": "暂无 Skills",
      "loading": "正在扫描...",
      "error": "扫描失败"
    },
    "detail": {
      "files": "文件",
      "size": "大小",
      "hash": "哈希",
      "preview": "预览"
    }
  }
}
```

## 错误处理

- Rust 端所有命令返回 `Result<T, String>`，错误信息通过 `Err(String)` 传递
- 前端 invoke 用 try/catch 捕获，使用 Naive UI 的 `useMessage()` 显示错误通知
- 操作类命令（安装/更新/卸载）在执行前弹出确认对话框

## 参考代码映射

| 新项目文件 | 旧项目参考 |
|-----------|-----------|
| `src-tauri/src/models/skills.rs` | `field-skill-manage/src-tauri/src/models/skill.rs` |
| `src-tauri/src/services/skills_service.rs` | `field-skill-manage/src-tauri/src/services/skill_service.rs` |
| `src-tauri/src/commands/skills.rs` | `field-skill-manage/src-tauri/src/commands/skill.rs` |
| `src/plugins/skills/components/SkillCard.vue` | `field-skill-manage/src/components/common/SkillCard.vue` |
| `src/plugins/skills/views/SkillsMainView.vue` | `field-skill-manage/src/views/GlobalView.vue` |
