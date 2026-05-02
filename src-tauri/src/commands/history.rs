use crate::services::history_service::{self, OperationRecord};

/// Get operation history, optionally limited to the most recent N records
#[tauri::command]
pub async fn get_operation_history(
    limit: Option<usize>,
) -> Result<Vec<OperationRecord>, String> {
    let mut records = history_service::load_history()?;
    if let Some(limit) = limit {
        records.truncate(limit);
    }
    Ok(records)
}

/// Rollback a specific operation by its ID
#[tauri::command]
pub async fn rollback_operation(id: String) -> Result<String, String> {
    history_service::rollback_operation(&id)
}

/// Clear all operation history
#[tauri::command]
pub async fn clear_history() -> Result<(), String> {
    history_service::clear_history()
}
