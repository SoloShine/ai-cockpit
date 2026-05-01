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