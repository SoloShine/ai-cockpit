use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    #[serde(rename = "basePath")]
    pub base_path: String,
    pub enabled: bool,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub appearance: AppearanceSettings,
    pub agents: Vec<AgentConfig>,
    pub plugins: PluginSettings,
    #[serde(rename = "_meta")]
    pub meta: MetaSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: String,
    #[serde(rename = "fontSize")]
    pub font_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSettings {
    #[serde(rename = "disabledIds")]
    pub disabled_ids: Vec<String>,
    pub order: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaSettings {
    pub version: u32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("无法创建数据目录: {}", e))?;
    Ok(data_dir.join("settings.json"))
}

pub fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "claude-code".into(),
            name: "Claude Code".into(),
            agent_type: "claude-code".into(),
            base_path: ".claude/commands".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "cursor".into(),
            name: "Cursor".into(),
            agent_type: "cursor".into(),
            base_path: ".cursor/commands".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "windsurf".into(),
            name: "Windsurf".into(),
            agent_type: "windsurf".into(),
            base_path: ".windsurf/commands".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "opencode".into(),
            name: "OpenCode".into(),
            agent_type: "opencode".into(),
            base_path: ".opencode/commands".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "codex".into(),
            name: "Codex".into(),
            agent_type: "codex".into(),
            base_path: ".codex/commands".into(),
            enabled: true,
            is_custom: false,
        },
    ]
}

pub fn default_settings() -> AppSettings {
    AppSettings {
        appearance: AppearanceSettings {
            theme: "system".into(),
            language: "zh-CN".into(),
            font_size: 14,
        },
        agents: default_agents(),
        plugins: PluginSettings {
            disabled_ids: vec![],
            order: vec![],
        },
        meta: MetaSettings {
            version: 1,
            updated_at: chrono_now(),
        },
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

pub fn load_settings(app: &tauri::AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        let defaults = default_settings();
        save_settings(app, &defaults)?;
        return Ok(defaults);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("无法读取设置文件: {}", e))?;
    let settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("无法解析设置文件: {}", e))?;
    Ok(settings)
}

pub fn save_settings(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> Result<(), String> {
    let path = settings_path(app)?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("无法序列化设置: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("无法写入设置文件: {}", e))?;
    Ok(())
}
