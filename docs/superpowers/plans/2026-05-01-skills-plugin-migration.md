# Skill 管理插件迁移 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 field-skill-manage 的核心 Skill 管理功能迁移为 ai-cockpit 的标准插件，分两批交付（扫描展示 + 安装操作）。

**Architecture:** Rust 后端采用无状态纯函数设计，路径参数从前端传入；前端采用卡片式布局 + Agent Tab + 全局/项目 Tab，通过 `useSettingsStore()` 跨插件读取 Agent 配置。

**Tech Stack:** Tauri 2, Rust (serde, sha2), Vue 3, TypeScript, Naive UI, Pinia, vue-i18n

**Spec:** `docs/superpowers/specs/2026-05-01-skills-plugin-migration-design.md`

**Reference:** `D:\Project\field-skill-manage` — 旧项目源码，特别是 `src-tauri/src/services/skill_service.rs`（扫描逻辑）和 `src-tauri/src/models/skill.rs`（数据模型）

---

## File Map

### New files — Rust backend

| File | Responsibility |
|------|---------------|
| `src-tauri/src/models/skills.rs` | Skill 数据模型（SkillInfo, ScanResult, FileEntry 等） |
| `src-tauri/src/services/skills_service.rs` | Skill 业务逻辑（扫描、哈希、文件树、安装、卸载） |
| `src-tauri/src/commands/skills.rs` | Skill IPC 命令（11 个 Tauri command） |

### New files — Frontend plugin

| File | Responsibility |
|------|---------------|
| `src/plugins/skills/index.ts` | Plugin 注册入口（CockpitPlugin + PluginHooks） |
| `src/plugins/skills/types.ts` | TypeScript 类型定义 |
| `src/plugins/skills/store.ts` | Pinia store（skill 状态管理） |
| `src/plugins/skills/composables.ts` | 公共 API 导出 |
| `src/plugins/skills/i18n/zh-CN.json` | 中文翻译 |
| `src/plugins/skills/i18n/en-US.json` | 英文翻译 |
| `src/plugins/skills/views/SkillsMainView.vue` | 主视图（范围 Tab + Agent Tab + Skill 列表） |
| `src/plugins/skills/components/ScopeTabs.vue` | 全局/项目范围切换 |
| `src/plugins/skills/components/AgentTabs.vue` | Agent 切换标签 |
| `src/plugins/skills/components/SkillCard.vue` | 单个 Skill 卡片 |
| `src/plugins/skills/components/SkillList.vue` | Skill 列表容器（卡片网格） |
| `src/plugins/skills/components/EmptyState.vue` | 空状态/引导提示 |
| `src/plugins/skills/components/BatchActionBar.vue` | 批量操作状态栏 |

### Modified files

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | 添加 `sha2` 依赖 |
| `src-tauri/src/commands/mod.rs` | 添加 `pub mod skills;` |
| `src-tauri/src/services/mod.rs` | 添加 `pub mod skills_service;` |
| `src-tauri/src/models/mod.rs` | 添加 `pub mod skills;` |
| `src-tauri/src/lib.rs` | 注册 skills 命令到 invoke_handler |
| `src/main.ts` | 注册 skills 插件 |

---

## Task 1: Add sha2 dependency to Cargo.toml

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add sha2 crate**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 段末尾添加：

```toml
sha2 = "0.10"
```

- [ ] **Step 2: Verify Cargo resolves**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src-tauri && cargo check 2>&1 | tail -5`
Expected: 编译通过（现有代码不依赖 sha2，不会出错）

- [ ] **Step 3: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src-tauri/Cargo.toml
git commit -m "chore: add sha2 dependency for skill hash calculation"
```

---

## Task 2: Create Rust data models

**Files:**
- Create: `src-tauri/src/models/skills.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: Create models/skills.rs**

创建 `src-tauri/src/models/skills.rs`，内容如下：

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill 元信息（从 SKILL.md frontmatter 或 skills.json 解析）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
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
    pub content_hash: String,
    pub last_modified: Option<String>,
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

- [ ] **Step 2: Update models/mod.rs**

修改 `src-tauri/src/models/mod.rs`，添加新模块：

```rust
pub mod skills;
```

注意：原文件只有一行注释 `// Shared data models...`，替换为上面的 `pub mod skills;`。如果该文件有其他 `pub mod` 行，在末尾追加即可。

- [ ] **Step 3: Verify compilation**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src-tauri && cargo check 2>&1 | tail -5`
Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src-tauri/src/models/skills.rs src-tauri/src/models/mod.rs
git commit -m "feat(skills): add Rust data models for skill plugin"
```

---

## Task 3: Create Rust service layer — scanning functions

**Files:**
- Create: `src-tauri/src/services/skills_service.rs`
- Modify: `src-tauri/src/services/mod.rs`

**Reference:** `D:\Project\field-skill-manage\src-tauri\src\services\skill_service.rs` — 扫描逻辑参考 `list_installed_skills`、`parse_skill_frontmatter`、`build_local_skill_meta` 函数

- [ ] **Step 1: Create services/skills_service.rs**

创建 `src-tauri/src/services/skills_service.rs`：

```rust
use crate::models::skills::*;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

/// 扫描指定目录下的所有 Skill
pub fn scan_skills(base_dir: &str, agent_id: &str, scope: SkillScope) -> Result<ScanResult, String> {
    let dir = Path::new(base_dir);
    if !dir.exists() {
        return Ok(ScanResult {
            agent_id: agent_id.to_string(),
            scope,
            skills: vec![],
            total: 0,
        });
    }

    let mut skills = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("无法读取目录 {}: {}", base_dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let path = entry.path();

        // 跳过隐藏文件和非 Skill 文件
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();
        if name.starts_with('.') || name == "skills.json" || name == "skillbase.json" {
            continue;
        }

        if let Some(skill_info) = build_skill_info(&path, &name)? {
            skills.push(skill_info);
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    let total = skills.len();

    Ok(ScanResult {
        agent_id: agent_id.to_string(),
        scope,
        skills,
        total,
    })
}

/// 构建 SkillInfo（支持文件和目录两种形式）
fn build_skill_info(path: &Path, name: &str) -> Result<Option<SkillInfo>, String> {
    if path.is_file() {
        build_file_skill_info(path, name)
    } else if path.is_dir() {
        build_dir_skill_info(path, name)
    } else {
        Ok(None)
    }
}

/// 单文件形式的 Skill（如 .md 文件）
fn build_file_skill_info(path: &Path, name: &str) -> Result<Option<SkillInfo>, String> {
    // 只处理 .md 文件
    if !name.ends_with(".md") {
        return Ok(None);
    }

    let meta = fs::metadata(path).map_err(|e| format!("无法读取文件元数据: {}", e))?;
    let content = fs::read_to_string(path).unwrap_or_default();
    let hash = calculate_hash(&content);

    let skill_name = name.trim_end_matches(".md").to_string();
    let skill_meta = parse_skill_meta(&content);

    Ok(Some(SkillInfo {
        name: skill_name,
        path: path.to_string_lossy().to_string(),
        is_file: true,
        has_skill_md: true,
        meta: skill_meta,
        file_count: 1,
        size_bytes: meta.len(),
        content_hash: hash,
        last_modified: get_modified_time(&meta),
        source_agent_id: None,
    }))
}

/// 目录形式的 Skill
fn build_dir_skill_info(path: &Path, name: &str) -> Result<Option<SkillInfo>, String> {
    let skill_md_path = path.join("SKILL.md");
    let has_skill_md = skill_md_path.exists();

    let mut file_count = 0usize;
    let mut total_size = 0u64;
    let mut hash_input = String::new();

    collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;

    let content_hash = calculate_hash(&hash_input);

    // 解析 SKILL.md 中的 meta（如果存在）
    let meta = if has_skill_md {
        let content = fs::read_to_string(&skill_md_path).unwrap_or_default();
        parse_skill_meta(&content)
    } else {
        None
    };

    Ok(Some(SkillInfo {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        is_file: false,
        has_skill_md,
        meta,
        file_count,
        size_bytes: total_size,
        content_hash,
        last_modified: None,
        source_agent_id: None,
    }))
}

/// 递归收集目录统计信息
fn collect_dir_stats(
    dir: &Path,
    file_count: &mut usize,
    total_size: &mut u64,
    hash_input: &mut String,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("无法读取目录: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            *file_count += 1;
            let meta = fs::metadata(&path).unwrap_or_default();
            *total_size += meta.len();
            let content = fs::read_to_string(&path).unwrap_or_default();
            hash_input.push_str(&content);
        } else if path.is_dir() {
            collect_dir_stats(&path, file_count, total_size, hash_input)?;
        }
    }
    Ok(())
}

/// 计算内容的 SHA256 哈希
pub fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// 获取文件修改时间的字符串表示
fn get_modified_time(meta: &fs::Metadata) -> Option<String> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let secs = d.as_secs();
            // 简单格式化为 YYYY-MM-DD（近似）
            let days = secs / 86400;
            let year = 1970 + (days / 365);
            let month = ((days % 365) / 30) + 1;
            let day = (days % 30) + 1;
            format!("{}-{:02}-{:02}", year, month, day)
        })
}

/// 解析 SKILL.md 内容提取元信息
fn parse_skill_meta(content: &str) -> Option<SkillMeta> {
    // 尝试解析 YAML frontmatter（--- 包裹的部分）
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let rest = &trimmed[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];

    let mut name = String::new();
    let mut description = String::new();
    let mut version = None;
    let mut author = None;
    let mut tags = vec![];

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("version:") {
            version = Some(val.trim().trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("author:") {
            author = Some(val.trim().trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("tags:") {
            // 简单解析 [tag1, tag2] 格式
            let inner = val.trim().trim_start_matches('[').trim_end_matches(']');
            tags = inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(SkillMeta {
        name,
        description,
        version,
        author,
        tags,
        dependencies: vec![],
    })
}

/// 构建文件树
pub fn build_file_tree(path: &str) -> Result<Vec<FileEntry>, String> {
    let root = Path::new(path);
    if !root.exists() {
        return Ok(vec![]);
    }

    if root.is_file() {
        let meta = fs::metadata(root).map_err(|e| format!("无法读取文件: {}", e))?;
        return Ok(vec![FileEntry {
            name: root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            path: root.to_string_lossy().to_string(),
            is_dir: false,
            size: meta.len(),
            children: vec![],
        }]);
    }

    let mut entries = Vec::new();
    build_file_tree_recursive(root, &mut entries)?;
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
    });
    Ok(entries)
}

fn build_file_tree_recursive(dir: &Path, result: &mut Vec<FileEntry>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("无法读取目录: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = fs::metadata(&path).unwrap_or_default();

        if path.is_dir() {
            let mut children = Vec::new();
            build_file_tree_recursive(&path, &mut children)?;
            children.sort_by(|a, b| {
                b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
            });
            result.push(FileEntry {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: true,
                size: meta.len(),
                children,
            });
        } else {
            result.push(FileEntry {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: false,
                size: meta.len(),
                children: vec![],
            });
        }
    }
    Ok(())
}

/// 安装 Skill（将源复制到目标路径）
pub fn install_skill(source: &str, target_path: &str) -> Result<OperationResult, String> {
    let src = Path::new(source);
    let dst = Path::new(target_path);

    if !src.exists() {
        return Err(format!("源路径不存在: {}", source));
    }

    if src.is_file() {
        // 单文件安装：复制文件到目标目录
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
        }
        fs::copy(src, dst).map_err(|e| format!("复制文件失败: {}", e))?;
    } else {
        // 目录安装：递归复制
        copy_dir_recursive(src, dst)?;
    }

    Ok(OperationResult {
        success: true,
        message: format!("已安装: {}", src.file_name().unwrap_or_default().to_string_lossy()),
        affected_paths: vec![target_path.to_string()],
    })
}

/// 更新 Skill（等同于覆盖安装）
pub fn update_skill(source: &str, target_path: &str) -> Result<OperationResult, String> {
    // 先删除目标（如果存在），再安装
    let dst = Path::new(target_path);
    if dst.exists() {
        if dst.is_dir() {
            fs::remove_dir_all(dst).map_err(|e| format!("删除旧版本失败: {}", e))?;
        } else {
            fs::remove_file(dst).map_err(|e| format!("删除旧版本失败: {}", e))?;
        }
    }
    install_skill(source, target_path)
}

/// 卸载 Skill（删除指定路径）
pub fn uninstall_skill(skill_path: &str) -> Result<OperationResult, String> {
    let path = Path::new(skill_path);
    if !path.exists() {
        return Err(format!("路径不存在: {}", skill_path));
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("删除目录失败: {}", e))?;
    } else {
        fs::remove_file(path).map_err(|e| format!("删除文件失败: {}", e))?;
    }

    Ok(OperationResult {
        success: true,
        message: format!("已卸载: {}", path.file_name().unwrap_or_default().to_string_lossy()),
        affected_paths: vec![skill_path.to_string()],
    })
}

/// 批量执行操作
pub fn batch_operate(operations: Vec<SkillOperation>) -> Vec<OperationResult> {
    operations
        .into_iter()
        .map(|op| {
            let result = match op.operation_type {
                OperationType::Install => install_skill(&op.source, &op.target_path),
                OperationType::Update => update_skill(&op.source, &op.target_path),
                OperationType::Uninstall => uninstall_skill(&op.target_path),
            };
            result.unwrap_or_else(|e| OperationResult {
                success: false,
                message: e,
                affected_paths: vec![],
            })
        })
        .collect()
}

/// 验证 Skill 完整性（比对哈希）
pub fn verify_skill_integrity(skill_path: &str, expected_hash: &str) -> Result<bool, String> {
    let path = Path::new(skill_path);
    if !path.exists() {
        return Ok(false);
    }

    let actual_hash = if path.is_file() {
        let content = fs::read_to_string(path).map_err(|e| format!("无法读取文件: {}", e))?;
        calculate_hash(&content)
    } else {
        let mut hash_input = String::new();
        let mut file_count = 0usize;
        let mut total_size = 0u64;
        collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;
        calculate_hash(&hash_input)
    };

    Ok(actual_hash == expected_hash)
}

/// 递归复制目录
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("无法创建目录: {}", e))?;
    let entries = fs::read_dir(src).map_err(|e| format!("无法读取源目录: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}

/// 获取项目概览
pub fn get_projects_overview(
    project_paths: Vec<String>,
    agent_ids: Vec<String>,
    global_paths: HashMap<String, String>,
    project_patterns: HashMap<String, String>,
) -> Result<Vec<ProjectOverview>, String> {
    let mut result = Vec::new();

    for project_path in project_paths {
        let project = Path::new(&project_path);
        let project_name = project
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut agent_skills_count = HashMap::new();

        for agent_id in &agent_ids {
            let pattern = match project_patterns.get(agent_id) {
                Some(p) => p.replace("{project}", &project_path),
                None => continue,
            };

            let pattern_path = Path::new(&pattern);
            if pattern_path.exists() {
                let scan = scan_skills(&pattern, agent_id, SkillScope::Project)?;
                agent_skills_count.insert(agent_id.clone(), scan.total);
            } else {
                agent_skills_count.insert(agent_id.clone(), 0);
            }
        }

        result.push(ProjectOverview {
            project_path,
            project_name,
            agent_skills_count,
        });
    }

    Ok(result)
}
```

- [ ] **Step 2: Update services/mod.rs**

修改 `src-tauri/src/services/mod.rs`，追加新模块：

```rust
pub mod settings_service;
pub mod skills_service;
```

- [ ] **Step 3: Verify compilation**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src-tauri/src/services/skills_service.rs src-tauri/src/services/mod.rs
git commit -m "feat(skills): add Rust service layer for skill scanning and operations"
```

---

## Task 4: Create Rust commands and register

**Files:**
- Create: `src-tauri/src/commands/skills.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create commands/skills.rs**

创建 `src-tauri/src/commands/skills.rs`：

```rust
use crate::models::skills::*;
use crate::services::skills_service;
use std::collections::HashMap;

#[tauri::command]
pub fn scan_global_skills(agent_id: String, global_path: String) -> Result<ScanResult, String> {
    // global_path 是相对于 HOME 的路径，需展开为绝对路径
    let home = dirs_home();
    let full_path = if global_path.starts_with('/') || global_path.starts_with('\\') || global_path.contains(':') {
        global_path.clone()
    } else {
        format!("{}/{}", home, global_path)
    };
    skills_service::scan_skills(&full_path, &agent_id, SkillScope::Global)
}

#[tauri::command]
pub fn scan_project_skills(
    agent_id: String,
    project_path: String,
    project_dir: String,
) -> Result<ScanResult, String> {
    let full_path = format!("{}/{}", project_path, project_dir);
    skills_service::scan_skills(&full_path, &agent_id, SkillScope::Project)
}

#[tauri::command]
pub fn get_skill_file_tree(skill_path: String) -> Result<Vec<FileEntry>, String> {
    skills_service::build_file_tree(&skill_path)
}

#[tauri::command]
pub fn read_skill_file(file_path: String) -> Result<String, String> {
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("无法读取文件 {}: {}", file_path, e))
}

#[tauri::command]
pub fn calculate_skill_hash(skill_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&skill_path);
    if !path.exists() {
        return Err(format!("路径不存在: {}", skill_path));
    }

    if path.is_file() {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        Ok(skills_service::calculate_hash(&content))
    } else {
        let mut hash_input = String::new();
        let mut file_count = 0usize;
        let mut total_size = 0u64;
        skills_service::collect_dir_stats(path, &mut file_count, &mut total_size, &mut hash_input)?;
        Ok(skills_service::calculate_hash(&hash_input))
    }
}

#[tauri::command]
pub fn get_projects_overview(
    project_paths: Vec<String>,
    agent_ids: Vec<String>,
    global_paths: HashMap<String, String>,
    project_patterns: HashMap<String, String>,
) -> Result<Vec<ProjectOverview>, String> {
    skills_service::get_projects_overview(project_paths, agent_ids, global_paths, project_patterns)
}

#[tauri::command]
pub fn install_skill(source: String, target_path: String) -> Result<OperationResult, String> {
    skills_service::install_skill(&source, &target_path)
}

#[tauri::command]
pub fn update_skill(source: String, target_path: String) -> Result<OperationResult, String> {
    skills_service::update_skill(&source, &target_path)
}

#[tauri::command]
pub fn uninstall_skill(skill_path: String) -> Result<OperationResult, String> {
    skills_service::uninstall_skill(&skill_path)
}

#[tauri::command]
pub fn batch_operate(operations: Vec<SkillOperation>) -> Result<Vec<OperationResult>, String> {
    Ok(skills_service::batch_operate(operations))
}

#[tauri::command]
pub fn verify_skill_integrity(
    skill_path: String,
    expected_hash: String,
) -> Result<bool, String> {
    skills_service::verify_skill_integrity(&skill_path, &expected_hash)
}

fn dirs_home() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}
```

**注意**：`collect_dir_stats` 是 private 函数。需要在 `skills_service.rs` 中将其改为 `pub`。

修改 `src-tauri/src/services/skills_service.rs` 中 `collect_dir_stats` 的可见性：

```rust
// 原来：fn collect_dir_stats(
// 改为：
pub fn collect_dir_stats(
```

- [ ] **Step 2: Update commands/mod.rs**

修改 `src-tauri/src/commands/mod.rs`，追加：

```rust
pub mod core;
pub mod settings;
pub mod skills;
```

- [ ] **Step 3: Update lib.rs to register commands**

修改 `src-tauri/src/lib.rs`，在 `invoke_handler` 宏中注册所有 skills 命令。完整替换为：

```rust
mod commands;
mod models;
mod services;

use commands::settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Core
            commands::core::get_app_version,
            // Settings
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::get_data_dir,
            commands::settings::open_in_explorer,
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
        .run(tauri::generate_context!())
        .expect("error while running AI Cockpit");
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过，无 warning 或 error

- [ ] **Step 5: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src-tauri/src/commands/skills.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/services/skills_service.rs
git commit -m "feat(skills): add Rust IPC commands and register in Tauri builder"
```

---

## Task 5: Create frontend types and i18n

**Files:**
- Create: `src/plugins/skills/types.ts`
- Create: `src/plugins/skills/i18n/zh-CN.json`
- Create: `src/plugins/skills/i18n/en-US.json`

- [ ] **Step 1: Create plugin directory structure**

Run: `mkdir -p D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src/plugins/skills/{views,components,i18n}`

- [ ] **Step 2: Create types.ts**

创建 `src/plugins/skills/types.ts`：

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
```

- [ ] **Step 3: Create i18n/zh-CN.json**

创建 `src/plugins/skills/i18n/zh-CN.json`：

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
      "batch": "批量操作",
      "detail": "详情"
    },
    "status": {
      "empty": "暂无 Skills",
      "emptyHint": "当前 Agent 目录下未发现 Skill 文件",
      "loading": "正在扫描...",
      "error": "扫描失败",
      "selected": "已选择 {count} 个",
      "total": "共 {count} 个 Skills"
    },
    "card": {
      "files": "{count} 个文件",
      "noMeta": "无元信息",
      "hash": "哈希"
    },
    "empty": {
      "noPath": "路径未配置",
      "noPathHint": "请前往设置页面配置当前 Agent 的 Skill 路径",
      "goSettings": "前往设置",
      "noSkills": "暂无 Skills",
      "noSkillsHint": "当前目录下未发现 Skill 文件",
      "scanError": "扫描出错",
      "retry": "重试"
    },
    "detail": {
      "title": "Skill 详情",
      "files": "文件列表",
      "size": "大小",
      "path": "路径"
    },
    "confirm": {
      "uninstall": "确认卸载",
      "uninstallMsg": "确定要卸载 Skill \"{name}\" 吗？此操作不可恢复。",
      "batchUninstall": "确认批量卸载",
      "batchUninstallMsg": "确定要卸载选中的 {count} 个 Skill 吗？"
    }
  }
}
```

- [ ] **Step 4: Create i18n/en-US.json**

创建 `src/plugins/skills/i18n/en-US.json`：

```json
{
  "skills": {
    "title": "Skill Manager",
    "scope": {
      "global": "Global Skills",
      "project": "Project Skills"
    },
    "actions": {
      "install": "Install",
      "update": "Update",
      "uninstall": "Uninstall",
      "batch": "Batch Actions",
      "detail": "Detail"
    },
    "status": {
      "empty": "No Skills",
      "emptyHint": "No skill files found in current agent directory",
      "loading": "Scanning...",
      "error": "Scan failed",
      "selected": "{count} selected",
      "total": "{count} Skills total"
    },
    "card": {
      "files": "{count} files",
      "noMeta": "No metadata",
      "hash": "Hash"
    },
    "empty": {
      "noPath": "Path not configured",
      "noPathHint": "Please configure the Skill path for this agent in Settings",
      "goSettings": "Go to Settings",
      "noSkills": "No Skills Found",
      "noSkillsHint": "No skill files found in current directory",
      "scanError": "Scan Error",
      "retry": "Retry"
    },
    "detail": {
      "title": "Skill Detail",
      "files": "Files",
      "size": "Size",
      "path": "Path"
    },
    "confirm": {
      "uninstall": "Confirm Uninstall",
      "uninstallMsg": "Are you sure you want to uninstall skill \"{name}\"? This cannot be undone.",
      "batchUninstall": "Confirm Batch Uninstall",
      "batchUninstallMsg": "Are you sure you want to uninstall {count} selected skills?"
    }
  }
}
```

- [ ] **Step 5: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/types.ts src/plugins/skills/i18n/
git commit -m "feat(skills): add frontend types and i18n translations"
```

---

## Task 6: Create Pinia store

**Files:**
- Create: `src/plugins/skills/store.ts`

- [ ] **Step 1: Create store.ts**

创建 `src/plugins/skills/store.ts`：

```typescript
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/plugins/settings/store";
import type {
  SkillInfo,
  SkillScope,
  ScanResult,
  SkillOperation,
  OperationResult,
} from "./types";

export const useSkillsStore = defineStore("skills", () => {
  // 当前状态
  const currentAgentId = ref<string>("claude-code");
  const currentScope = ref<SkillScope>("global");
  const currentProjectPath = ref<string>("");

  // 缓存：agentId → ScanResult
  const globalSkills = ref<Map<string, ScanResult>>(new Map());
  const projectSkills = ref<Map<string, ScanResult>>(new Map());

  // 选中状态
  const selectedSkills = ref<Set<string>>(new Set());

  // UI 状态
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 计算属性：当前显示的 Skill 列表
  const currentSkills = computed<SkillInfo[]>(() => {
    const map =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    const result = map.get(currentAgentId.value);
    return result?.skills ?? [];
  });

  // 计算属性：可用的 Agent 列表（enabled）
  const availableAgents = computed(() => {
    const settings = useSettingsStore();
    return settings.agents.filter((a) => a.enabled);
  });

  // 获取当前 Agent 的配置
  function getCurrentAgentConfig() {
    const settings = useSettingsStore();
    return settings.agents.find((a) => a.id === currentAgentId.value);
  }

  // 扫描指定 Agent 的 Skills
  async function scanSkills(agentId: string, scope: SkillScope) {
    const settings = useSettingsStore();
    const agent = settings.agents.find((a) => a.id === agentId);
    if (!agent) return;

    loading.value = true;
    error.value = null;

    try {
      let result: ScanResult;
      if (scope === "global") {
        result = await invoke<ScanResult>("scan_global_skills", {
          agentId,
          globalPath: agent.globalPath,
        });
        globalSkills.value.set(agentId, result);
      } else {
        if (!currentProjectPath.value) return;
        result = await invoke<ScanResult>("scan_project_skills", {
          agentId,
          projectPath: currentProjectPath.value,
          projectDir: agent.projectPath,
        });
        projectSkills.value.set(agentId, result);
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  // 扫描所有 Agent
  async function scanAllAgents(scope: SkillScope) {
    const agents = availableAgents.value;
    await Promise.allSettled(agents.map((a) => scanSkills(a.id, scope)));
  }

  // 切换 Agent 并扫描
  async function switchAgent(agentId: string) {
    currentAgentId.value = agentId;
    selectedSkills.value.clear();

    const map =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    if (!map.has(agentId)) {
      await scanSkills(agentId, currentScope.value);
    }
  }

  // 切换范围
  async function switchScope(scope: SkillScope) {
    currentScope.value = scope;
    selectedSkills.value.clear();
    await scanAllAgents(scope);
  }

  // 安装 Skill
  async function installSkill(
    source: string,
    targetPath: string,
  ): Promise<OperationResult> {
    const result = await invoke<OperationResult>("install_skill", {
      source,
      targetPath,
    });
    await scanSkills(currentAgentId.value, currentScope.value);
    return result;
  }

  // 更新 Skill
  async function updateSkill(
    source: string,
    targetPath: string,
  ): Promise<OperationResult> {
    const result = await invoke<OperationResult>("update_skill", {
      source,
      targetPath,
    });
    await scanSkills(currentAgentId.value, currentScope.value);
    return result;
  }

  // 卸载 Skill
  async function uninstallSkill(skillPath: string): Promise<OperationResult> {
    const result = await invoke<OperationResult>("uninstall_skill", {
      skillPath,
    });
    await scanSkills(currentAgentId.value, currentScope.value);
    return result;
  }

  // 批量操作
  async function batchOperate(
    operations: SkillOperation[],
  ): Promise<OperationResult[]> {
    const results = await invoke<OperationResult[]>("batch_operate", {
      operations,
    });
    await scanSkills(currentAgentId.value, currentScope.value);
    return results;
  }

  // 选择管理
  function toggleSelect(skillName: string) {
    if (selectedSkills.value.has(skillName)) {
      selectedSkills.value.delete(skillName);
    } else {
      selectedSkills.value.add(skillName);
    }
  }

  function selectAll() {
    for (const skill of currentSkills.value) {
      selectedSkills.value.add(skill.name);
    }
  }

  function clearSelection() {
    selectedSkills.value.clear();
  }

  return {
    // State
    currentAgentId,
    currentScope,
    currentProjectPath,
    globalSkills,
    projectSkills,
    selectedSkills,
    loading,
    error,
    // Computed
    currentSkills,
    availableAgents,
    // Actions
    getCurrentAgentConfig,
    scanSkills,
    scanAllAgents,
    switchAgent,
    switchScope,
    installSkill,
    updateSkill,
    uninstallSkill,
    batchOperate,
    toggleSelect,
    selectAll,
    clearSelection,
  };
});
```

- [ ] **Step 2: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/store.ts
git commit -m "feat(skills): add Pinia store with scan, selection, and operation methods"
```

---

## Task 7: Create plugin entry and register

**Files:**
- Create: `src/plugins/skills/index.ts`
- Create: `src/plugins/skills/composables.ts`
- Modify: `src/main.ts`

- [ ] **Step 1: Create composables.ts**

创建 `src/plugins/skills/composables.ts`：

```typescript
export { useSkillsStore } from "./store";
```

- [ ] **Step 2: Create index.ts**

创建 `src/plugins/skills/index.ts`：

```typescript
import { RocketOutline } from "@vicons/ionicons5";
import type { PluginModule } from "@/core/plugin";
import { useSkillsStore } from "./store";
import i18n from "@/core/i18n";

import zhCN from "./i18n/zh-CN.json";
import enUS from "./i18n/en-US.json";

const plugin: PluginModule = {
  default: {
    id: "skills",
    name: "Skill 管理",
    icon: RocketOutline,
    routes: [
      {
        path: "/skills",
        name: "skills",
        component: () => import("./views/SkillsMainView.vue"),
        meta: { pluginId: "skills" },
      },
    ],
    navItems: [
      {
        routeName: "skills",
        label: "Skill 管理",
        icon: RocketOutline,
      },
    ],
    order: 10,
  },
  hooks: {
    async onInit() {
      i18n.global.mergeLocaleMessage("zh-CN", zhCN);
      i18n.global.mergeLocaleMessage("en-US", enUS);
    },
    async onActivate() {
      const store = useSkillsStore();
      await store.scanAllAgents(store.currentScope);
    },
  },
};

export default plugin;
```

- [ ] **Step 3: Register in main.ts**

修改 `src/main.ts`，在 `import settingsModule` 之后添加 skills 插件导入和注册。

在 `import settingsModule from "./plugins/settings";` 之后添加：

```typescript
import skillsModule from "./plugins/skills";
```

在 `pluginRegistry.register(settingsModule);` 之后添加：

```typescript
pluginRegistry.register(skillsModule);
```

- [ ] **Step 4: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/index.ts src/plugins/skills/composables.ts src/main.ts
git commit -m "feat(skills): register skill plugin in app bootstrap"
```

---

## Task 8: Create UI components — ScopeTabs, AgentTabs, EmptyState

**Files:**
- Create: `src/plugins/skills/components/ScopeTabs.vue`
- Create: `src/plugins/skills/components/AgentTabs.vue`
- Create: `src/plugins/skills/components/EmptyState.vue`

- [ ] **Step 1: Create ScopeTabs.vue**

创建 `src/plugins/skills/components/ScopeTabs.vue`：

```vue
<script setup lang="ts">
import { NTabs, NTabPane } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";

const { t } = useI18n();
const store = useSkillsStore();

function handleChange(value: string) {
  store.switchScope(value as "global" | "project");
}
</script>

<template>
  <NTabs
    :value="store.currentScope"
    type="segment"
    size="small"
    @update:value="handleChange"
  >
    <NTabPane name="global" :tab="t('skills.scope.global')" />
    <NTabPane name="project" :tab="t('skills.scope.project')" />
  </NTabs>
</template>
```

- [ ] **Step 2: Create AgentTabs.vue**

创建 `src/plugins/skills/components/AgentTabs.vue`：

```vue
<script setup lang="ts">
import { NScrollbar, NTab, NTabs } from "naive-ui";
import { useSkillsStore } from "../store";

const store = useSkillsStore();

function handleChange(agentId: string) {
  store.switchAgent(agentId);
}
</script>

<template>
  <NScrollbar x-scrollable>
    <NTabs
      :value="store.currentAgentId"
      type="line"
      size="small"
      @update:value="handleChange"
    >
      <NTab
        v-for="agent in store.availableAgents"
        :key="agent.id"
        :name="agent.id"
      >
        {{ agent.name }}
      </NTab>
    </NTabs>
  </NScrollbar>
</template>
```

- [ ] **Step 3: Create EmptyState.vue**

创建 `src/plugins/skills/components/EmptyState.vue`：

```vue
<script setup lang="ts">
import { NResult, NButton } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useSkillsStore } from "../store";

defineProps<{
  type: "noPath" | "noSkills" | "error";
}>();

const { t } = useI18n();
const router = useRouter();
const store = useSkillsStore();
</script>

<template>
  <div style="display: flex; justify-content: center; padding: 48px 0">
    <NResult
      v-if="type === 'noPath'"
      status="info"
      :title="t('skills.empty.noPath')"
      :description="t('skills.empty.noPathHint')"
    >
      <template #footer>
        <NButton @click="router.push({ name: 'settings' })">
          {{ t("skills.empty.goSettings") }}
        </NButton>
      </template>
    </NResult>
    <NResult
      v-else-if="type === 'noSkills'"
      status="info"
      :title="t('skills.empty.noSkills')"
      :description="t('skills.empty.noSkillsHint')"
    />
    <NResult
      v-else
      status="error"
      :title="t('skills.empty.scanError')"
      :description="store.error ?? ''"
    >
      <template #footer>
        <NButton @click="store.scanSkills(store.currentAgentId, store.currentScope)">
          {{ t("skills.empty.retry") }}
        </NButton>
      </template>
    </NResult>
  </div>
</template>
```

- [ ] **Step 4: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/components/ScopeTabs.vue src/plugins/skills/components/AgentTabs.vue src/plugins/skills/components/EmptyState.vue
git commit -m "feat(skills): add ScopeTabs, AgentTabs, and EmptyState components"
```

---

## Task 9: Create UI components — SkillCard, SkillList, BatchActionBar

**Files:**
- Create: `src/plugins/skills/components/SkillCard.vue`
- Create: `src/plugins/skills/components/SkillList.vue`
- Create: `src/plugins/skills/components/BatchActionBar.vue`

- [ ] **Step 1: Create SkillCard.vue**

创建 `src/plugins/skills/components/SkillCard.vue`：

```vue
<script setup lang="ts">
import {
  NCard,
  NCheckbox,
  NSpace,
  NText,
  NTag,
  NButton,
  NTooltip,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import type { SkillInfo } from "../types";

defineProps<{ skill: SkillInfo }>();
defineEmits<{
  uninstall: [skill: SkillInfo];
}>();
const { t } = useI18n();
const store = useSkillsStore();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
</script>

<template>
  <NCard size="small" hoverable style="cursor: pointer">
    <template #header>
      <NSpace align="center" :wrap="false">
        <NCheckbox
          :checked="store.selectedSkills.has(skill.name)"
          @update:checked="store.toggleSelect(skill.name)"
          @click.stop
        />
        <NText strong>{{ skill.meta?.name ?? skill.name }}</NText>
        <NTag v-if="skill.meta?.version" size="small" type="info">
          v{{ skill.meta.version }}
        </NTag>
      </NSpace>
    </template>
    <NText depth="3" style="font-size: 13px">
      {{ skill.meta?.description ?? t("skills.card.noMeta") }}
    </NText>
    <template #action>
      <NSpace justify="space-between" align="center">
        <NSpace :size="12">
          <NText depth="3" style="font-size: 12px">
            {{ t("skills.card.files", { count: skill.fileCount }) }}
            · {{ formatSize(skill.sizeBytes) }}
          </NText>
        </NSpace>
        <NTooltip>
          <template #trigger>
            <NButton
              size="tiny"
              type="error"
              quaternary
              @click.stop="$emit('uninstall', skill)"
            >
              {{ t("skills.actions.uninstall") }}
            </NButton>
          </template>
          {{ skill.path }}
        </NTooltip>
      </NSpace>
    </template>
  </NCard>
</template>
```

- [ ] **Step 2: Create SkillList.vue**

创建 `src/plugins/skills/components/SkillList.vue`：

```vue
<script setup lang="ts">
import { NGrid, NGridItem, NSpin, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import SkillCard from "./SkillCard.vue";
import EmptyState from "./EmptyState.vue";
import type { SkillInfo } from "../types";

const { t } = useI18n();
const store = useSkillsStore();

function handleUninstall(skill: SkillInfo) {
  store.uninstallSkill(skill.path);
}
</script>

<template>
  <NSpin :show="store.loading">
    <template v-if="!store.loading && store.currentSkills.length === 0">
      <EmptyState
        :type="store.error ? 'error' : 'noSkills'"
      />
    </template>
    <template v-else>
      <NText depth="3" style="margin-bottom: 12px; display: block; font-size: 13px">
        {{ t("skills.status.total", { count: store.currentSkills.length }) }}
      </NText>
      <NGrid :cols="2" :x-gap="12" :y-gap="12" responsive="screen">
        <NGridItem
          v-for="skill in store.currentSkills"
          :key="skill.name"
        >
          <SkillCard :skill="skill" @uninstall="handleUninstall" />
        </NGridItem>
      </NGrid>
    </template>
  </NSpin>
</template>
```

- [ ] **Step 3: Create BatchActionBar.vue**

创建 `src/plugins/skills/components/BatchActionBar.vue`：

```vue
<script setup lang="ts">
import { NSpace, NText, NButton } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";

const { t } = useI18n();
const store = useSkillsStore();
</script>

<template>
  <NSpace
    v-if="store.selectedSkills.size > 0"
    align="center"
    justify="space-between"
    style="
      position: sticky;
      bottom: 0;
      padding: 12px 16px;
      background: var(--n-color);
      border-top: 1px solid var(--n-border-color);
      border-radius: 0 0 var(--n-border-radius) var(--n-border-radius);
    "
  >
    <NText>
      {{ t("skills.status.selected", { count: store.selectedSkills.size }) }}
    </NText>
    <NSpace>
      <NButton size="small" @click="store.clearSelection()">
        {{ t("skills.actions.batch") }}
      </NButton>
      <NButton size="small" type="error" @click="store.clearSelection()">
        {{ t("skills.actions.uninstall") }}
      </NButton>
    </NSpace>
  </NSpace>
</template>
```

- [ ] **Step 4: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/components/SkillCard.vue src/plugins/skills/components/SkillList.vue src/plugins/skills/components/BatchActionBar.vue
git commit -m "feat(skills): add SkillCard, SkillList, and BatchActionBar components"
```

---

## Task 10: Create SkillsMainView

**Files:**
- Create: `src/plugins/skills/views/SkillsMainView.vue`

- [ ] **Step 1: Create SkillsMainView.vue**

创建 `src/plugins/skills/views/SkillsMainView.vue`：

```vue
<script setup lang="ts">
import { onMounted } from "vue";
import { NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import ScopeTabs from "../components/ScopeTabs.vue";
import AgentTabs from "../components/AgentTabs.vue";
import SkillList from "../components/SkillList.vue";
import BatchActionBar from "../components/BatchActionBar.vue";
import EmptyState from "../components/EmptyState.vue";

const { t } = useI18n();
const store = useSkillsStore();

onMounted(() => {
  const agentConfig = store.getCurrentAgentConfig();
  if (!agentConfig || (!agentConfig.globalPath && !agentConfig.projectPath)) {
    return;
  }
  store.scanAllAgents(store.currentScope);
});
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace align="center" justify="space-between">
        <NText strong style="font-size: 18px">
          {{ t("skills.title") }}
        </NText>
      </NSpace>

      <ScopeTabs />

      <AgentTabs />

      <SkillList />
    </NSpace>

    <BatchActionBar />
  </div>
</template>
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe && npm run build 2>&1 | tail -20`
Expected: 构建成功，无 TypeScript 错误

- [ ] **Step 3: Commit**

```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add src/plugins/skills/views/SkillsMainView.vue
git commit -m "feat(skills): add SkillsMainView with scope/agent tabs and skill list"
```

---

## Task 11: Final verification and end-to-end check

**Files:** None (verification only)

- [ ] **Step 1: Full Rust build check**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe/src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过

- [ ] **Step 2: Full frontend build check**

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe && npm run build 2>&1 | tail -20`
Expected: 构建成功

- [ ] **Step 3: Verify plugin appears in sidebar**

启动 dev server 验证 Skill 管理出现在侧边栏，点击可切换到 skills 页面：

Run: `cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe && npm run tauri dev &`

手动验证：
1. 侧边栏出现"Skill 管理"菜单项
2. 点击后显示 Agent Tab（10 个 Agent）
3. 切换 Agent 能扫描对应目录（如果目录存在）
4. 卡片正确展示 Skill 信息

- [ ] **Step 4: Final commit if any fixups needed**

如果有构建修复：
```bash
cd D:/Project/ai-cockpit/.claude/worktrees/elegant-mendel-f4d0fe
git add -A
git commit -m "fix(skills): address build issues from integration"
```
