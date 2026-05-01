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
    #[serde(rename = "globalPath")]
    pub global_path: String,
    #[serde(rename = "projectPath")]
    pub project_path: String,
    pub enabled: bool,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub cache_path: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub appearance: AppearanceSettings,
    pub agents: Vec<AgentConfig>,
    pub repos: Vec<RepoConfig>,
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
            global_path: ".claude".into(),
            project_path: ".claude".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "cursor".into(),
            name: "Cursor".into(),
            agent_type: "cursor".into(),
            global_path: ".cursor".into(),
            project_path: ".cursor".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "windsurf".into(),
            name: "Windsurf".into(),
            agent_type: "windsurf".into(),
            global_path: ".codeium/windsurf".into(),
            project_path: ".windsurf".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "opencode".into(),
            name: "OpenCode".into(),
            agent_type: "opencode".into(),
            global_path: ".config/opencode".into(),
            project_path: ".opencode".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "codex".into(),
            name: "OpenAI Codex".into(),
            agent_type: "codex".into(),
            global_path: ".codex".into(),
            project_path: ".codex".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "cline".into(),
            name: "Cline".into(),
            agent_type: "cline".into(),
            global_path: "Documents/Cline/Rules".into(),
            project_path: ".clinerules".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "augment".into(),
            name: "Augment".into(),
            agent_type: "augment".into(),
            global_path: ".augment".into(),
            project_path: ".augment".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "aider".into(),
            name: "Aider".into(),
            agent_type: "aider".into(),
            global_path: ".aider".into(),
            project_path: ".aider".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "copilot".into(),
            name: "GitHub Copilot".into(),
            agent_type: "copilot".into(),
            global_path: "github/copilot".into(),
            project_path: ".github".into(),
            enabled: true,
            is_custom: false,
        },
        AgentConfig {
            id: "trae".into(),
            name: "Trae".into(),
            agent_type: "trae".into(),
            global_path: ".trae".into(),
            project_path: ".trae".into(),
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
        repos: vec![],
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
    let mut settings: AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("无法解析设置文件: {}", e))?;

    // Migration: strip stale "/skills" suffix — settings now store the agent base dir
    let mut dirty = false;
    for agent in &mut settings.agents {
        if agent.global_path.ends_with("/skills") {
            agent.global_path = agent.global_path.trim_end_matches("/skills").to_string();
            dirty = true;
        }
        if agent.project_path.ends_with("/skills") {
            agent.project_path = agent.project_path.trim_end_matches("/skills").to_string();
            dirty = true;
        }
    }
    if dirty {
        let _ = save_settings(app, &settings);
    }

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
