use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// A single operation record in the history log
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub id: String,
    pub operation_type: OperationType,
    pub skill_name: String,
    pub target_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_after: Option<String>,
    pub can_rollback: bool,
    pub rolled_back: bool,
}

/// Type of operation performed
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

const MAX_RECORDS: usize = 200;

/// Get the path to the history file
fn history_file_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot determine application data directory".to_string())?;
    let app_dir = data_dir.join("ai-cockpit");
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    Ok(app_dir.join("history.json"))
}

/// Load history records from disk. Returns empty vec if file not found.
pub fn load_history() -> Result<Vec<OperationRecord>, String> {
    let path = history_file_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read history file: {}", e))?;
    let records: Vec<OperationRecord> = serde_json::from_str(&content)
        .unwrap_or_else(|e| {
            eprintln!("[history_service] Warning: failed to parse history file, resetting: {}", e);
            Vec::new()
        });
    Ok(records)
}

/// Save history records to disk
pub fn save_history(records: &[OperationRecord]) -> Result<(), String> {
    let path = history_file_path()?;
    let content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write history file: {}", e))?;
    Ok(())
}

/// Record a new operation and persist it
pub fn record_operation(
    operation_type: OperationType,
    skill_name: String,
    target_path: String,
    source_path: Option<String>,
    version_before: Option<String>,
    version_after: Option<String>,
) -> Result<OperationRecord, String> {
    let mut records = load_history()?;

    // Determine if rollback is possible
    let can_rollback = match &operation_type {
        OperationType::Install => true, // Can delete installed files
        OperationType::Uninstall => source_path.is_some(), // Can re-copy from source
        OperationType::Update => source_path.is_some(),    // Can re-copy from source
    };

    let record = OperationRecord {
        id: Uuid::new_v4().to_string(),
        operation_type,
        skill_name,
        target_path,
        source_path,
        timestamp: Utc::now().to_rfc3339(),
        version_before,
        version_after,
        can_rollback,
        rolled_back: false,
    };

    // Insert at the beginning (newest first)
    records.insert(0, record.clone());

    // Truncate to max records
    records.truncate(MAX_RECORDS);

    save_history(&records)?;
    Ok(record)
}

/// Rollback a specific operation by ID
pub fn rollback_operation(id: &str) -> Result<String, String> {
    let mut records = load_history()?;

    // Find the record index first
    let idx = records
        .iter()
        .position(|r| r.id == id)
        .ok_or_else(|| format!("Operation record not found: {}", id))?;

    // Validate
    if records[idx].rolled_back {
        return Err("This operation has already been rolled back".to_string());
    }

    if !records[idx].can_rollback {
        return Err("This operation cannot be rolled back".to_string());
    }

    // Extract data needed for the rollback (to avoid borrow conflicts)
    let operation_type = records[idx].operation_type.clone();
    let target_path_str = records[idx].target_path.clone();
    let source_path_str = records[idx].source_path.clone();
    let skill_name = records[idx].skill_name.clone();

    let target = Path::new(&target_path_str);

    match operation_type {
        OperationType::Install => {
            // Rollback install: delete the installed files
            if target.exists() {
                if target.is_file() {
                    fs::remove_file(target)
                        .map_err(|e| format!("Failed to delete file: {}", e))?;
                } else {
                    fs::remove_dir_all(target)
                        .map_err(|e| format!("Failed to delete directory: {}", e))?;
                }
            }
        }
        OperationType::Uninstall | OperationType::Update => {
            // Rollback uninstall/update: re-copy from source_path (best effort)
            let source_path = source_path_str
                .ok_or_else(|| "No source path available for rollback".to_string())?;
            let source = Path::new(&source_path);

            if !source.exists() {
                return Err(format!(
                    "Source path no longer exists: {}. Cannot rollback.",
                    source_path
                ));
            }

            // Remove target if it exists (e.g., for update rollback)
            if target.exists() {
                if target.is_file() {
                    fs::remove_file(target)
                        .map_err(|e| format!("Failed to remove existing file: {}", e))?;
                } else {
                    fs::remove_dir_all(target)
                        .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
                }
            }

            // Create parent directory if needed
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            // Copy from source to target
            if source.is_file() {
                fs::copy(source, target)
                    .map_err(|e| format!("Failed to restore file: {}", e))?;
            } else {
                copy_dir_recursive(source, target)?;
            }
        }
    }

    // Mark as rolled back and save
    records[idx].rolled_back = true;
    save_history(&records)?;

    Ok(format!(
        "Successfully rolled back {} of '{}'",
        match operation_type {
            OperationType::Install => "installation",
            OperationType::Update => "update",
            OperationType::Uninstall => "uninstall",
        },
        skill_name
    ))
}

/// Clear all history records
pub fn clear_history() -> Result<(), String> {
    let path = history_file_path()?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to delete history file: {}", e))?;
    }
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;
    }

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_file() {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {} to {}: {}", src_path.display(), dst_path.display(), e))?;
        } else if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
