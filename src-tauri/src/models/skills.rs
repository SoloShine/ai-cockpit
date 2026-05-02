use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata parsed from SKILL.md frontmatter or skills.json
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<HashMap<String, String>>,
}

/// Information about a discovered skill
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub name: String,
    pub path: String,
    pub is_file: bool,
    pub has_skill_md: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<SkillMeta>,
    pub file_count: u64,
    pub size_bytes: u64,
    pub content_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_agent_id: Option<String>,
}

/// Scope of skill installation
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SkillScope {
    Global,
    Project,
}

/// Result of scanning a directory for skills
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub agent_id: String,
    pub scope: SkillScope,
    pub skills: Vec<SkillInfo>,
    pub total: u64,
}

/// File entry in a directory tree
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    #[serde(default)]
    pub children: Vec<FileEntry>,
}

/// Status difference between source and target
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiffStatus {
    Same,
    Modified,
    Added,
    Removed,
}

/// Difference between two skill versions
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiff {
    pub name: String,
    pub source_hash: String,
    pub target_hash: String,
    pub status: DiffStatus,
    #[serde(default)]
    pub changed_files: Vec<String>,
}

/// File-level diff information
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub diff_type: DiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Result of a skill operation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub affected_paths: Vec<String>,
}

/// Type of operation to perform
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

/// A single skill operation request
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillOperation {
    pub operation_type: OperationType,
    pub source: String,
    pub target_path: String,
}

/// Overview of skills across projects
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverview {
    pub project_path: String,
    pub project_name: String,
    #[serde(default)]
    pub agent_skills_count: HashMap<String, u64>,
}

/// Detailed breakdown of skills in a project
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project_path: String,
    pub project_name: String,
    pub agents: Vec<AgentSkillInfo>,
}

/// Skill info for a specific agent in a project
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkillInfo {
    pub agent_id: String,
    pub skills: Vec<SkillInfo>,
    pub total: u64,
}

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

/// Migration scan result for a single skill
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrateSkillItem {
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub status: String, // "newTarget" | "sameContent" | "differentVersion" | "contentDiffers"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A single migration request
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrateRequest {
    pub name: String,
    pub source_path: String,
    pub target_path: String,
    pub resolution: String, // "Skip" or "Overwrite"
}

/// Migration result summary
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrateResult {
    #[serde(default)]
    pub migrated: Vec<String>,
    #[serde(default)]
    pub skipped: Vec<String>,
    #[serde(default)]
    pub failed: Vec<MigrateFailedItem>,
}

/// A single failed migration item
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrateFailedItem {
    pub name: String,
    pub error: String,
}
