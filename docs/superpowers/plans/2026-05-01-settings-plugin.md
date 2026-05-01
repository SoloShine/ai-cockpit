# 设置插件实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现完整的设置插件，包含外观与语言、Agent 配置、插件管理、关于与维护四个 Tab。

**Architecture:** 设置作为标准 CockpitPlugin 注册，通过 Pinia store 管理状态，Rust 后端读写本地 JSON 文件。主题/语言通过 NConfigProvider 和 vue-i18n 全局生效。

**Tech Stack:** Vue 3 + TypeScript + Naive UI + Pinia + vue-i18n + Tauri 2 + Rust + serde_json

---

## 文件结构

### 新建文件

| 文件 | 职责 |
|------|------|
| `src/core/i18n/index.ts` | vue-i18n 实例 + 核心消息 |
| `src/plugins/settings/index.ts` | 插件注册入口 |
| `src/plugins/settings/types.ts` | AppSettings、AgentConfig 类型 |
| `src/plugins/settings/store.ts` | Pinia settingsStore |
| `src/plugins/settings/views/SettingsView.vue` | Tab 布局外壳 |
| `src/plugins/settings/panels/AppearancePanel.vue` | 外观与语言面板 |
| `src/plugins/settings/panels/AgentPanel.vue` | Agent 配置面板 |
| `src/plugins/settings/panels/PluginPanel.vue` | 插件管理面板 |
| `src/plugins/settings/panels/AboutPanel.vue` | 关于与维护面板 |
| `src/plugins/settings/components/ThemeSelector.vue` | 主题选择卡片 |
| `src/plugins/settings/components/AgentCard.vue` | 单个 Agent 卡片 |
| `src/plugins/settings/components/AddAgentDialog.vue` | 添加 Agent 对话框 |
| `src/plugins/settings/components/PluginDetail.vue` | 插件详情展开 |
| `src/plugins/settings/i18n/zh-CN.json` | 中文 i18n |
| `src/plugins/settings/i18n/en-US.json` | 英文 i18n |
| `src-tauri/src/commands/settings.rs` | Tauri IPC 命令 |
| `src-tauri/src/services/settings_service.rs` | 设置文件读写服务 |

### 修改文件

| 文件 | 修改内容 |
|------|---------|
| `src/main.ts` | 注册 i18n、注册设置插件 |
| `src/App.vue` | 包裹 NConfigProvider，应用主题 |
| `src/core/plugin/registry.ts` | 添加 getDisabledIds / setPluginEnabled |
| `src/core/layout/Sidebar.vue` | 根据插件禁用状态过滤导航 |
| `src/stores/plugin.ts` | 添加 disabledIds 响应 |
| `src-tauri/src/commands/mod.rs` | 添加 settings 模块 |
| `src-tauri/src/services/mod.rs` | 添加 settings_service 模块 |
| `src-tauri/src/lib.rs` | 注册 settings 命令 |

---

## Task 1: App Shell — NConfigProvider + vue-i18n

为设置插件的主题和语言功能提供全局支持。修改 App.vue 包裹 NConfigProvider，创建 i18n 基础设施。

**Files:**
- Create: `src/core/i18n/index.ts`
- Modify: `src/main.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 创建 i18n 基础模块**

```typescript
// src/core/i18n/index.ts
import { createI18n } from "vue-i18n";

const i18n = createI18n({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "en-US",
  messages: {},
});

export default i18n;
```

- [ ] **Step 2: 修改 main.ts 注册 i18n 和设置插件**

```typescript
// src/main.ts
import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import router from "./router";
import App from "./App.vue";
import { pluginRegistry } from "./core/plugin";
import { usePluginStore } from "./stores/plugin";
import i18n from "./core/i18n";

// 注册内置插件
import settingsModule from "./plugins/settings";
pluginRegistry.register(settingsModule);

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(i18n);
app.use(router);
app.use(naive);

usePluginStore().refresh();

app.mount("#app");
```

- [ ] **Step 3: 修改 App.vue 包裹 NConfigProvider**

```vue
<!-- src/App.vue -->
<script setup lang="ts">
import { computed } from "vue";
import { NConfigProvider, darkTheme, zhCN, dateZhCN, enUS, dateEnUS } from "naive-ui";
import AppLayout from "@/core/layout/AppLayout.vue";
import { useSettingsStore } from "@/plugins/settings/store";

const settingsStore = useSettingsStore();

const theme = computed(() =>
  settingsStore.appearance.theme === "dark" ? darkTheme : null
);

const localeMap = {
  "zh-CN": { locale: zhCN, dateLocale: dateZhCN },
  "en-US": { locale: enUS, dateLocale: dateEnUS },
};

const naiveLocale = computed(() => localeMap[settingsStore.appearance.language].locale);
const naiveDateLocale = computed(() => localeMap[settingsStore.appearance.language].dateLocale);
</script>

<template>
  <NConfigProvider :theme="theme" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <AppLayout />
  </NConfigProvider>
</template>
```

- [ ] **Step 4: 验证**

运行: `npm run tauri dev`
预期: 应用正常启动，侧边栏和欢迎页正常显示（此时设置插件还未创建，会报错，先忽略）

- [ ] **Step 5: 提交**

```bash
git add src/core/i18n/index.ts src/main.ts src/App.vue
git commit -m "feat: add i18n setup and NConfigProvider to app shell"
```

---

## Task 2: 设置插件类型定义 + i18n 资源

创建 TypeScript 类型和设置插件的 i18n 资源文件。

**Files:**
- Create: `src/plugins/settings/types.ts`
- Create: `src/plugins/settings/i18n/zh-CN.json`
- Create: `src/plugins/settings/i18n/en-US.json`

- [ ] **Step 1: 创建类型定义**

```typescript
// src/plugins/settings/types.ts
export interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "zh-CN" | "en-US";
  fontSize: number;
}

export interface AgentConfig {
  id: string;
  name: string;
  type: string;
  basePath: string;
  enabled: boolean;
  isCustom: boolean;
}

export interface PluginSettings {
  disabledIds: string[];
  order: string[];
}

export interface AppSettings {
  appearance: AppearanceSettings;
  agents: AgentConfig[];
  plugins: PluginSettings;
  _meta: {
    version: number;
    updatedAt: string;
  };
}
```

- [ ] **Step 2: 创建 composables.ts 公共 API**

其他插件通过这些 composable 获取公共配置，不直接依赖 store 内部结构：

```typescript
// src/plugins/settings/composables.ts
import { computed } from "vue";
import { useSettingsStore } from "./store";
import type { AgentConfig, AppearanceSettings } from "./types";

/** 获取已启用的 Agent 列表（含路径） */
export function useAgentPaths() {
  const store = useSettingsStore();
  const enabledAgents = computed(() =>
    store.agents.filter((a) => a.enabled)
  );
  const getAgentById = (id: string) => store.agents.find((a) => a.id === id);
  return { enabledAgents, getAgentById, allAgents: computed(() => store.agents) };
}

/** 获取外观配置（主题、语言、字号） */
export function useAppAppearance() {
  const store = useSettingsStore();
  return {
    theme: computed(() => store.appearance.theme),
    language: computed(() => store.appearance.language),
    fontSize: computed(() => store.appearance.fontSize),
  };
}

/** 查询插件是否启用 */
export function usePluginEnabled(pluginId: string) {
  const store = useSettingsStore();
  return computed(() => !store.plugins.disabledIds.includes(pluginId));
}
```

- [ ] **Step 3: 创建中文 i18n 资源**

```json
{
  "settings": {
    "title": "设置",
    "tabs": {
      "appearance": "外观与语言",
      "agents": "Agent 配置",
      "plugins": "插件管理",
      "about": "关于"
    },
    "appearance": {
      "theme": "主题",
      "themeLight": "亮色",
      "themeDark": "暗色",
      "themeSystem": "跟随系统",
      "language": "语言",
      "fontSize": "字体大小"
    },
    "agents": {
      "title": "Agent 列表",
      "addCustom": "添加自定义 Agent",
      "name": "名称",
      "type": "类型标识",
      "basePath": "配置路径",
      "browse": "浏览",
      "enabled": "已启用",
      "delete": "删除",
      "deleteConfirm": "确定要删除此 Agent 配置吗？",
      "builtIn": "内置",
      "custom": "自定义",
      "nameRequired": "请输入 Agent 名称",
      "typeRequired": "请输入类型标识",
      "pathRequired": "请选择或输入配置路径",
      "addSuccess": "Agent 添加成功",
      "deleteSuccess": "Agent 已删除"
    },
    "plugins": {
      "title": "已安装插件",
      "enabled": "已启用",
      "disabled": "已禁用",
      "noSettings": "此插件暂无专属设置",
      "routes": "注册路由",
      "settingsPluginProtected": "设置插件不可禁用"
    },
    "about": {
      "version": "版本",
      "dataDir": "数据目录",
      "openInExplorer": "在资源管理器中打开",
      "checkUpdates": "检查更新",
      "openLogs": "打开日志目录",
      "techStack": "技术栈"
    }
  }
}
```

- [ ] **Step 4: 创建英文 i18n 资源**

```json
{
  "settings": {
    "title": "Settings",
    "tabs": {
      "appearance": "Appearance",
      "agents": "Agent Config",
      "plugins": "Plugins",
      "about": "About"
    },
    "appearance": {
      "theme": "Theme",
      "themeLight": "Light",
      "themeDark": "Dark",
      "themeSystem": "System",
      "language": "Language",
      "fontSize": "Font Size"
    },
    "agents": {
      "title": "Agent List",
      "addCustom": "Add Custom Agent",
      "name": "Name",
      "type": "Type Identifier",
      "basePath": "Config Path",
      "browse": "Browse",
      "enabled": "Enabled",
      "delete": "Delete",
      "deleteConfirm": "Are you sure you want to delete this agent?",
      "builtIn": "Built-in",
      "custom": "Custom",
      "nameRequired": "Agent name is required",
      "typeRequired": "Type identifier is required",
      "pathRequired": "Config path is required",
      "addSuccess": "Agent added successfully",
      "deleteSuccess": "Agent deleted"
    },
    "plugins": {
      "title": "Installed Plugins",
      "enabled": "Enabled",
      "disabled": "Disabled",
      "noSettings": "This plugin has no settings",
      "routes": "Registered Routes",
      "settingsPluginProtected": "Settings plugin cannot be disabled"
    },
    "about": {
      "version": "Version",
      "dataDir": "Data Directory",
      "openInExplorer": "Open in Explorer",
      "checkUpdates": "Check for Updates",
      "openLogs": "Open Logs Directory",
      "techStack": "Tech Stack"
    }
  }
}
```

- [ ] **Step 5: 提交**

```bash
git add src/plugins/settings/types.ts src/plugins/settings/composables.ts src/plugins/settings/i18n/
git commit -m "feat(settings): add type definitions and i18n resources"
```

---

## Task 3: Rust 后端 — 设置服务 + 命令

创建 Rust 端的设置文件读写服务和 Tauri IPC 命令。

**Files:**
- Create: `src-tauri/src/services/settings_service.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 settings_service.rs**

```rust
// src-tauri/src/services/settings_service.rs
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
```

- [ ] **Step 2: 创建 settings_commands.rs**

```rust
// src-tauri/src/commands/settings.rs
use tauri::State;
use crate::services::settings_service::{self, AppSettings};

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    settings_service::load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    settings_service::save_settings(&app, &settings)
}
```

- [ ] **Step 3: 更新 mod.rs 文件**

在 `src-tauri/src/commands/mod.rs` 添加：
```rust
pub mod core;
pub mod settings;
```

在 `src-tauri/src/services/mod.rs` 添加：
```rust
pub mod settings_service;
```

- [ ] **Step 4: 更新 lib.rs 注册命令**

```rust
// src-tauri/src/lib.rs
mod commands;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::core::get_app_version,
            commands::settings::load_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AI Cockpit");
}
```

- [ ] **Step 5: 验证 Rust 编译**

运行: `cd src-tauri && cargo check`
预期: 编译成功无错误

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/
git commit -m "feat(settings): add Rust settings service and IPC commands"
```

---

## Task 4: Pinia Settings Store

创建设置的状态管理 store，包含加载、保存、默认值逻辑。

**Files:**
- Create: `src/plugins/settings/store.ts`

- [ ] **Step 1: 创建 settingsStore**

```typescript
// src/plugins/settings/store.ts
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, AppearanceSettings, AgentConfig, PluginSettings } from "./types";

const DEFAULT_APPEARANCE: AppearanceSettings = {
  theme: "system",
  language: "zh-CN",
  fontSize: 14,
};

const DEFAULT_AGENTS: AgentConfig[] = [
  { id: "claude-code", name: "Claude Code", type: "claude-code", basePath: ".claude/commands", enabled: true, isCustom: false },
  { id: "cursor", name: "Cursor", type: "cursor", basePath: ".cursor/commands", enabled: true, isCustom: false },
  { id: "windsurf", name: "Windsurf", type: "windsurf", basePath: ".windsurf/commands", enabled: true, isCustom: false },
  { id: "opencode", name: "OpenCode", type: "opencode", basePath: ".opencode/commands", enabled: true, isCustom: false },
  { id: "codex", name: "Codex", type: "codex", basePath: ".codex/commands", enabled: true, isCustom: false },
];

const DEFAULT_PLUGINS: PluginSettings = {
  disabledIds: [],
  order: [],
};

export const useSettingsStore = defineStore("settings", () => {
  const loaded = ref(false);
  const appearance = ref<AppearanceSettings>({ ...DEFAULT_APPEARANCE });
  const agents = ref<AgentConfig[]>([...DEFAULT_AGENTS]);
  const plugins = ref<PluginSettings>({ ...DEFAULT_PLUGINS });

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  async function load() {
    try {
      const settings = await invoke<AppSettings>("load_settings");
      appearance.value = settings.appearance;
      agents.value = settings.agents;
      plugins.value = settings.plugins;
    } catch (e) {
      console.warn("[SettingsStore] 加载设置失败，使用默认值:", e);
    } finally {
      loaded.value = true;
    }
  }

  async function save() {
    if (!loaded.value) return;
    const settings: AppSettings = {
      appearance: appearance.value,
      agents: agents.value,
      plugins: plugins.value,
      _meta: { version: 1, updatedAt: new Date().toISOString() },
    };
    try {
      await invoke("save_settings", { settings });
    } catch (e) {
      console.error("[SettingsStore] 保存设置失败:", e);
    }
  }

  function scheduleSave() {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => save(), 500);
  }

  // 自动保存：监听所有设置变更
  watch([appearance, agents, plugins], () => scheduleSave(), { deep: true });

  function updateTheme(theme: AppearanceSettings["theme"]) {
    appearance.value.theme = theme;
  }

  function updateLanguage(language: AppearanceSettings["language"]) {
    appearance.value.language = language;
  }

  function updateFontSize(size: number) {
    appearance.value.fontSize = size;
  }

  function addAgent(agent: AgentConfig) {
    agents.value.push(agent);
  }

  function removeAgent(id: string) {
    agents.value = agents.value.filter((a) => a.id !== id);
  }

  function updateAgent(id: string, updates: Partial<AgentConfig>) {
    const idx = agents.value.findIndex((a) => a.id === id);
    if (idx !== -1) {
      agents.value[idx] = { ...agents.value[idx], ...updates };
    }
  }

  function togglePlugin(pluginId: string, enabled: boolean) {
    if (enabled) {
      plugins.value.disabledIds = plugins.value.disabledIds.filter((id) => id !== pluginId);
    } else {
      if (!plugins.value.disabledIds.includes(pluginId)) {
        plugins.value.disabledIds.push(pluginId);
      }
    }
  }

  function updatePluginOrder(order: string[]) {
    plugins.value.order = order;
  }

  return {
    loaded,
    appearance,
    agents,
    plugins,
    load,
    save,
    updateTheme,
    updateLanguage,
    updateFontSize,
    addAgent,
    removeAgent,
    updateAgent,
    togglePlugin,
    updatePluginOrder,
  };
});
```

- [ ] **Step 2: 提交**

```bash
git add src/plugins/settings/store.ts
git commit -m "feat(settings): add Pinia settings store with auto-save"
```

---

## Task 5: 插件注册入口

创建设置插件的注册入口文件，将插件接入应用。

**Files:**
- Create: `src/plugins/settings/index.ts`

- [ ] **Step 1: 创建插件入口**

```typescript
// src/plugins/settings/index.ts
import { SettingsOutline } from "@vicons/ionicons5";
import type { CockpitPlugin, PluginHooks, PluginModule } from "@/core/plugin";
import { useSettingsStore } from "./store";
import i18n from "@/core/i18n";

// 合并插件 i18n 消息
import zhCN from "./i18n/zh-CN.json";
import enUS from "./i18n/en-US.json";

const plugin: CockpitPlugin = {
  id: "settings",
  name: "设置",
  icon: SettingsOutline,
  routes: [
    {
      path: "/settings",
      name: "settings",
      component: () => import("./views/SettingsView.vue"),
      meta: { pluginId: "settings" },
    },
  ],
  navItems: [
    { routeName: "settings", label: "设置", icon: SettingsOutline },
  ],
  order: 999,
};

const hooks: PluginHooks = {
  async onInit() {
    // 合并 i18n 消息
    i18n.global.mergeLocaleMessage("zh-CN", zhCN);
    i18n.global.mergeLocaleMessage("en-US", enUS);

    // 加载设置
    const store = useSettingsStore();
    await store.load();

    // 应用语言设置
    i18n.global.locale.value = store.appearance.language;
  },
};

const settingsModule: PluginModule = {
  default: plugin,
  hooks,
};

export default settingsModule;
```

- [ ] **Step 2: 提交**

```bash
git add src/plugins/settings/index.ts
git commit -m "feat(settings): add plugin registration entry with i18n merge"
```

---

## Task 6: SettingsView 外壳 + AppearancePanel

创建设置页面的 Tab 布局和第一个面板（外观与语言）。

**Files:**
- Create: `src/plugins/settings/views/SettingsView.vue`
- Create: `src/plugins/settings/panels/AppearancePanel.vue`
- Create: `src/plugins/settings/components/ThemeSelector.vue`

- [ ] **Step 1: 创建 ThemeSelector 组件**

```vue
<!-- src/plugins/settings/components/ThemeSelector.vue -->
<script setup lang="ts">
import { NCard, NIcon, NText, useMessage } from "naive-ui";
import { SunnyOutline, MoonOutline, DesktopOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import type { AppearanceSettings } from "../types";

const props = defineProps<{ modelValue: AppearanceSettings["theme"] }>();
const emit = defineEmits<{ "update:modelValue": [value: AppearanceSettings["theme"]] }>();
const { t } = useI18n();

const themes: { value: AppearanceSettings["theme"]; icon: any; label: string }[] = [
  { value: "light", icon: SunnyOutline, label: t("settings.appearance.themeLight") },
  { value: "dark", icon: MoonOutline, label: t("settings.appearance.themeDark") },
  { value: "system", icon: DesktopOutline, label: t("settings.appearance.themeSystem") },
];
</script>

<template>
  <div style="display: flex; gap: 12px">
    <NCard
      v-for="theme in themes"
      :key="theme.value"
      hoverable
      :class="{ 'theme-card--active': modelValue === theme.value }"
      style="flex: 1; cursor: pointer; text-align: center"
      @click="emit('update:modelValue', theme.value)"
    >
      <NIcon size="32"><component :is="theme.icon" /></NIcon>
      <div style="margin-top: 8px">
        <NText>{{ theme.label }}</NText>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.theme-card--active {
  border-color: var(--n-color-target);
  box-shadow: 0 0 0 2px var(--n-color-target);
}
.theme-card--active :deep(.n-card__content) {
  background: var(--n-color-target);
  border-radius: var(--n-border-radius);
}
</style>
```

- [ ] **Step 2: 创建 AppearancePanel**

```vue
<!-- src/plugins/settings/panels/AppearancePanel.vue -->
<script setup lang="ts">
import { NForm, NFormItem, NSelect, NSlider, NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import ThemeSelector from "../components/ThemeSelector.vue";

const { t } = useI18n();
const store = useSettingsStore();

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
];
</script>

<template>
  <NForm label-placement="left" label-width="100">
    <NFormItem :label="t('settings.appearance.theme')">
      <ThemeSelector v-model="store.appearance.theme" />
    </NFormItem>
    <NFormItem :label="t('settings.appearance.language')">
      <NSelect
        :value="store.appearance.language"
        :options="languageOptions"
        style="width: 200px"
        @update:value="store.updateLanguage($event)"
      />
    </NFormItem>
    <NFormItem :label="t('settings.appearance.fontSize')">
      <NSpace align="center">
        <NSlider
          :value="store.appearance.fontSize"
          :min="12"
          :max="20"
          :step="1"
          style="width: 200px"
          @update:value="store.updateFontSize($event)"
        />
        <NText depth="3">{{ store.appearance.fontSize }}px</NText>
      </NSpace>
    </NFormItem>
  </NForm>
</template>
```

- [ ] **Step 3: 创建 SettingsView 外壳（支持插件挂载独立 Tab）**

SettingsView 从 pluginRegistry 收集所有带 `SettingsPanel` hook 的插件，为每个插件渲染独立 Tab。

```vue
<!-- src/plugins/settings/views/SettingsView.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { NTabs, NTabPane } from "naive-ui";
import { useI18n } from "vue-i18n";
import { pluginRegistry } from "@/core/plugin";
import AppearancePanel from "../panels/AppearancePanel.vue";
import AgentPanel from "../panels/AgentPanel.vue";
import PluginPanel from "../panels/PluginPanel.vue";
import AboutPanel from "../panels/AboutPanel.vue";

const { t } = useI18n();
const activeTab = ref("appearance");

// 收集所有带 SettingsPanel 的插件，为它们生成独立 Tab
const pluginSettingsTabs = computed(() => {
  return pluginRegistry.getAll()
    .filter((p) => {
      if (p.id === "settings") return false; // 排除设置插件自身
      const hooks = pluginRegistry.getHooks(p.id);
      return !!hooks?.SettingsPanel;
    })
    .map((p) => ({
      id: p.id,
      name: p.name,
      component: pluginRegistry.getHooks(p.id)!.SettingsPanel!,
    }));
});
</script>

<template>
  <div style="height: 100%">
    <NTabs
      v-model:value="activeTab"
      type="line"
      placement="left"
      style="height: 100%"
      :tabs-padding="24"
    >
      <NTabPane name="appearance" :tab="t('settings.tabs.appearance')">
        <AppearancePanel />
      </NTabPane>
      <NTabPane name="agents" :tab="t('settings.tabs.agents')">
        <AgentPanel />
      </NTabPane>
      <!-- 插件设置 Tab：动态挂载 -->
      <NTabPane
        v-for="tab in pluginSettingsTabs"
        :key="tab.id"
        :name="`plugin-${tab.id}`"
        :tab="tab.name"
      >
        <component :is="tab.component" />
      </NTabPane>
      <NTabPane name="plugins" :tab="t('settings.tabs.plugins')">
        <PluginPanel />
      </NTabPane>
      <NTabPane name="about" :tab="t('settings.tabs.about')">
        <AboutPanel />
      </NTabPane>
    </NTabs>
  </div>
</template>
```

Tab 顺序：外观 → Agent → [插件设置 Tab...] → 插件管理 → 关于。插件的设置 Tab 插入在"插件管理"之前，保持核心 Tab 在首尾。

- [ ] **Step 4: 提交**

```bash
git add src/plugins/settings/views/ src/plugins/settings/panels/AppearancePanel.vue src/plugins/settings/components/ThemeSelector.vue
git commit -m "feat(settings): add SettingsView shell and AppearancePanel"
```

---

## Task 7: AgentPanel — Agent 配置面板

创建 Agent 配置的完整 UI：列表展示、添加自定义、编辑路径、启用/禁用、删除。

**Files:**
- Create: `src/plugins/settings/panels/AgentPanel.vue`
- Create: `src/plugins/settings/components/AgentCard.vue`
- Create: `src/plugins/settings/components/AddAgentDialog.vue`

- [ ] **Step 1: 创建 AgentCard 组件**

```vue
<!-- src/plugins/settings/components/AgentCard.vue -->
<script setup lang="ts">
import { NCard, NSwitch, NButton, NInput, NTag, NSpace, NIcon, NPopconfirm } from "naive-ui";
import { FolderOpenOutline, TrashOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import type { AgentConfig } from "../types";

const props = defineProps<{ agent: AgentConfig }>();
const emit = defineEmits<{
  "update:agent": [updates: Partial<AgentConfig>];
  delete: [];
}>();
const { t } = useI18n();

async function browsePath() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    emit("update:agent", { basePath: selected });
  }
}
</script>

<template>
  <NCard size="small" style="margin-bottom: 12px">
    <template #header>
      <NSpace align="center">
        <span>{{ agent.name }}</span>
        <NTag :type="agent.isCustom ? 'info' : 'default'" size="small">
          {{ agent.isCustom ? t("settings.agents.custom") : t("settings.agents.builtIn") }}
        </NTag>
      </NSpace>
    </template>
    <NSpace vertical>
      <NSpace align="center">
        <NInput
          :value="agent.basePath"
          :placeholder="t('settings.agents.basePath')"
          style="flex: 1"
          @update:value="emit('update:agent', { basePath: $event })"
        />
        <NButton @click="browsePath">
          <template #icon><NIcon><FolderOpenOutline /></NIcon></template>
          {{ t("settings.agents.browse") }}
        </NButton>
      </NSpace>
      <NSpace justify="space-between" align="center">
        <NSwitch
          :value="agent.enabled"
          @update:value="emit('update:agent', { enabled: $event })"
        >
          <template #checked>{{ t("settings.agents.enabled") }}</template>
        </NSwitch>
        <NPopconfirm v-if="agent.isCustom" @positive-click="emit('delete')">
          <template #trigger>
            <NButton type="error" size="small" quaternary>
              <template #icon><NIcon><TrashOutline /></NIcon></template>
              {{ t("settings.agents.delete") }}
            </NButton>
          </template>
          {{ t("settings.agents.deleteConfirm") }}
        </NPopconfirm>
      </NSpace>
    </NSpace>
  </NCard>
</template>
```

- [ ] **Step 2: 创建 AddAgentDialog 组件**

```vue
<!-- src/plugins/settings/components/AddAgentDialog.vue -->
<script setup lang="ts">
import { ref } from "vue";
import {
  NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace, useMessage,
} from "naive-ui";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import type { AgentConfig } from "../types";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
  "update:show": [value: boolean];
  add: [agent: AgentConfig];
}>();

const { t } = useI18n();
const message = useMessage();

const name = ref("");
const agentType = ref("");
const basePath = ref("");

async function browsePath() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) basePath.value = selected;
}

function handleSubmit() {
  if (!name.value.trim()) { message.warning(t("settings.agents.nameRequired")); return; }
  if (!agentType.value.trim()) { message.warning(t("settings.agents.typeRequired")); return; }
  if (!basePath.value.trim()) { message.warning(t("settings.agents.pathRequired")); return; }

  const id = `custom-${Date.now()}`;
  emit("add", {
    id,
    name: name.value.trim(),
    type: agentType.value.trim(),
    basePath: basePath.value.trim(),
    enabled: true,
    isCustom: true,
  });

  name.value = "";
  agentType.value = "";
  basePath.value = "";
  emit("update:show", false);
  message.success(t("settings.agents.addSuccess"));
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)">
    <NCard
      style="width: 480px"
      :title="t('settings.agents.addCustom')"
      :bordered="false"
      size="medium"
      role="dialog"
      closable
      @close="emit('update:show', false)"
    >
      <NForm label-placement="left" label-width="100">
        <NFormItem :label="t('settings.agents.name')">
          <NInput v-model:value="name" />
        </NFormItem>
        <NFormItem :label="t('settings.agents.type')">
          <NInput v-model:value="agentType" placeholder="例如: my-agent" />
        </NFormItem>
        <NFormItem :label="t('settings.agents.basePath')">
          <NSpace style="width: 100%">
            <NInput v-model:value="basePath" style="flex: 1" />
            <NButton @click="browsePath">{{ t("settings.agents.browse") }}</NButton>
          </NSpace>
        </NFormItem>
      </NForm>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="emit('update:show', false)">取消</NButton>
          <NButton type="primary" @click="handleSubmit">确定</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
```

- [ ] **Step 3: 创建 AgentPanel**

```vue
<!-- src/plugins/settings/panels/AgentPanel.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { NButton, NSpace, NText, useMessage } from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import AgentCard from "../components/AgentCard.vue";
import AddAgentDialog from "../components/AddAgentDialog.vue";

const { t } = useI18n();
const store = useSettingsStore();
const message = useMessage();
const showAddDialog = ref(false);

function handleUpdateAgent(id: string, updates: Record<string, any>) {
  store.updateAgent(id, updates);
}

function handleDeleteAgent(id: string) {
  store.removeAgent(id);
  message.success(t("settings.agents.deleteSuccess"));
}
</script>

<template>
  <div>
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText strong style="font-size: 16px">{{ t("settings.agents.title") }}</NText>
      <NButton type="primary" @click="showAddDialog = true">
        <template #icon><AddOutline /></template>
        {{ t("settings.agents.addCustom") }}
      </NButton>
    </NSpace>

    <AgentCard
      v-for="agent in store.agents"
      :key="agent.id"
      :agent="agent"
      @update:agent="handleUpdateAgent(agent.id, $event)"
      @delete="handleDeleteAgent(agent.id)"
    />

    <AddAgentDialog
      :show="showAddDialog"
      @update:show="showAddDialog = $event"
      @add="store.addAgent($event)"
    />
  </div>
</template>
```

- [ ] **Step 4: 提交**

```bash
git add src/plugins/settings/panels/AgentPanel.vue src/plugins/settings/components/AgentCard.vue src/plugins/settings/components/AddAgentDialog.vue
git commit -m "feat(settings): add AgentPanel with CRUD and browse support"
```

---

## Task 8: PluginPanel — 插件管理面板

创建插件管理面板，支持启用/禁用、查看详情。插件专属设置现在通过 SettingsView 的独立 Tab 挂载，PluginDetail 只展示插件信息。拖拽排序暂用上下按钮替代。

**Files:**
- Create: `src/plugins/settings/panels/PluginPanel.vue`
- Create: `src/plugins/settings/components/PluginDetail.vue`
- Modify: `src/core/layout/Sidebar.vue` — 过滤已禁用插件

- [ ] **Step 1: 创建 PluginDetail 组件**

PluginDetail 只展示插件信息（路由、描述），插件专属设置由 SettingsView 动态 Tab 提供。

```vue
<!-- src/plugins/settings/components/PluginDetail.vue -->
<script setup lang="ts">
import { NDescriptions, NDescriptionsItem, NTag, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import type { CockpitPlugin } from "@/core/plugin";

const props = defineProps<{ plugin: CockpitPlugin }>();
const { t } = useI18n();
</script>

<template>
  <div style="padding: 12px 0">
    <NDescriptions label-placement="left" bordered :column="1" size="small">
      <NDescriptionsItem :label="t('settings.plugins.routes')">
        <NTag
          v-for="route in plugin.routes"
          :key="route.path"
          size="small"
          style="margin-right: 4px"
        >
          {{ route.path }}
        </NTag>
      </NDescriptionsItem>
    </NDescriptions>
  </div>
</template>
```

- [ ] **Step 2: 创建 PluginPanel**

```vue
<!-- src/plugins/settings/panels/PluginPanel.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { NCard, NSwitch, NSpace, NText, NIcon, NCollapse, NCollapseItem, NButton } from "naive-ui";
import { ChevronUpOutline, ChevronDownOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import { usePluginStore } from "@/stores/plugin";
import PluginDetail from "../components/PluginDetail.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const pluginStore = usePluginStore();
const expandedId = ref<string | null>(null);

function isEnabled(pluginId: string): boolean {
  return !settingsStore.plugins.disabledIds.includes(pluginId);
}

function toggleEnabled(pluginId: string, enabled: boolean) {
  if (pluginId === "settings") return; // 设置插件不可禁用
  settingsStore.togglePlugin(pluginId, enabled);
}

function moveUp(index: number) {
  const plugins = [...pluginStore.plugins];
  const id = plugins[index].id;
  const order = plugins.map((p) => p.id);
  if (index > 0) {
    [order[index], order[index - 1]] = [order[index - 1], order[index]];
    settingsStore.updatePluginOrder(order);
    pluginStore.refresh();
  }
}

function moveDown(index: number) {
  const plugins = [...pluginStore.plugins];
  const order = plugins.map((p) => p.id);
  if (index < plugins.length - 1) {
    [order[index], order[index + 1]] = [order[index + 1], order[index]];
    settingsStore.updatePluginOrder(order);
    pluginStore.refresh();
  }
}
</script>

<template>
  <div>
    <NText strong style="font-size: 16px; display: block; margin-bottom: 16px">
      {{ t("settings.plugins.title") }}
    </NText>

    <NCard
      v-for="(plugin, index) in pluginStore.plugins"
      :key="plugin.id"
      size="small"
      style="margin-bottom: 8px"
    >
      <NSpace align="center" justify="space-between">
        <NSpace align="center">
          <NIcon size="20"><component :is="plugin.icon" /></NIcon>
          <NText strong>{{ plugin.name }}</NText>
          <NText depth="3" v-if="plugin.description">{{ plugin.description }}</NText>
        </NSpace>
        <NSpace align="center">
          <NButton
            quaternary size="tiny"
            :disabled="index === 0"
            @click="moveUp(index)"
          >
            <template #icon><NIcon><ChevronUpOutline /></NIcon></template>
          </NButton>
          <NButton
            quaternary size="tiny"
            :disabled="index === pluginStore.plugins.length - 1"
            @click="moveDown(index)"
          >
            <template #icon><NIcon><ChevronDownOutline /></NIcon></template>
          </NButton>
          <NSwitch
            :value="isEnabled(plugin.id)"
            :disabled="plugin.id === 'settings'"
            @update:value="toggleEnabled(plugin.id, $event)"
          />
        </NSpace>
      </NSpace>

      <NCollapse v-model:expanded-names="expandedId">
        <NCollapseItem :name="plugin.id" title="">
          <PluginDetail :plugin="plugin" />
        </NCollapseItem>
      </NCollapse>
    </NCard>
  </div>
</template>
```

- [ ] **Step 3: 修改 Sidebar.vue 过滤已禁用插件**

在 `src/core/layout/Sidebar.vue` 的 `<script setup>` 中添加 import 和过滤逻辑。在现有 import 后添加：

```typescript
import { useSettingsStore } from "@/plugins/settings/store";

const settingsStore = useSettingsStore();
```

然后替换 `pluginMenuOptions` 计算属性为完整代码：

```typescript
const pluginMenuOptions = computed<MenuOption[]>(() =>
  pluginStore.navGroups
    .filter(({ pluginId }) => !settingsStore.plugins.disabledIds.includes(pluginId))
    .map(({ pluginId, items }) => {
      if (items.length === 1) {
        const item = items[0];
        return {
          label: item.label,
          key: item.routeName,
          icon: item.icon ? renderIcon(item.icon) : undefined,
        };
      }
      return {
        label: pluginStore.plugins.find((p) => p.id === pluginId)?.name ?? pluginId,
        key: pluginId,
        icon: renderIcon(
          pluginStore.plugins.find((p) => p.id === pluginId)?.icon ?? ""
        ),
        children: items.map((item) => ({
          label: item.label,
          key: item.routeName,
          icon: item.icon ? renderIcon(item.icon) : undefined,
        })),
      };
    })
);
```

- [ ] **Step 4: 提交**

```bash
git add src/plugins/settings/panels/PluginPanel.vue src/plugins/settings/components/PluginDetail.vue src/core/layout/Sidebar.vue
git commit -m "feat(settings): add PluginPanel with enable/disable and detail view"
```

---

## Task 9: AboutPanel — 关于与维护面板

创建关于面板，显示应用版本、数据目录、检查更新和日志目录。

**Files:**
- Create: `src/plugins/settings/panels/AboutPanel.vue`
- Modify: `src-tauri/src/commands/settings.rs` — 添加 open_data_dir 命令
- Modify: `src-tauri/src/lib.rs` — 注册新命令

- [ ] **Step 1: 添加 Rust 命令 — 获取数据目录路径 + 打开目录**

在 `src-tauri/src/commands/settings.rs` 末尾添加：

```rust
#[tauri::command]
pub fn get_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;
    Ok(data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("无法打开目录: {}", e))?;
    }
    Ok(())
}
```

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中添加注册：

```rust
.invoke_handler(tauri::generate_handler![
    commands::core::get_app_version,
    commands::settings::load_settings,
    commands::settings::save_settings,
    commands::settings::get_data_dir,
    commands::settings::open_in_explorer,
])
```

- [ ] **Step 2: 创建 AboutPanel**

```vue
<!-- src/plugins/settings/panels/AboutPanel.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NDescriptions, NDescriptionsItem, NButton, NSpace, NText, NTag, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const message = useMessage();

const version = ref("");
const dataDir = ref("");

onMounted(async () => {
  version.value = await invoke<string>("get_app_version");
  dataDir.value = await invoke<string>("get_data_dir");
});

async function openDataDir() {
  try {
    await invoke("open_in_explorer", { path: dataDir.value });
  } catch (e) {
    message.error("无法打开目录: " + e);
  }
}

async function openLogs() {
  // 日志目录暂时等同于数据目录
  await openDataDir();
}

function checkUpdates() {
  message.info("检查更新功能将在后续版本中实现");
}
</script>

<template>
  <div>
    <NDescriptions label-placement="left" bordered :column="1" title="AI Cockpit">
      <NDescriptionsItem :label="t('settings.about.version')">
        <NTag type="success">v{{ version }}</NTag>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.dataDir')">
        <NSpace align="center">
          <NText code>{{ dataDir }}</NText>
          <NButton size="tiny" @click="openDataDir">
            {{ t("settings.about.openInExplorer") }}
          </NButton>
        </NSpace>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.techStack')">
        <NSpace>
          <NTag>Tauri 2</NTag>
          <NTag>Vue 3</NTag>
          <NTag>TypeScript</NTag>
          <NTag>Naive UI</NTag>
          <NTag>Rust</NTag>
        </NSpace>
      </NDescriptionsItem>
    </NDescriptions>

    <NSpace style="margin-top: 16px">
      <NButton @click="checkUpdates">{{ t("settings.about.checkUpdates") }}</NButton>
      <NButton @click="openLogs">{{ t("settings.about.openLogs") }}</NButton>
    </NSpace>
  </div>
</template>
```

- [ ] **Step 3: 验证 Rust 编译**

运行: `cd src-tauri && cargo check`
预期: 编译成功

- [ ] **Step 4: 提交**

```bash
git add src/plugins/settings/panels/AboutPanel.vue src-tauri/src/
git commit -m "feat(settings): add AboutPanel with version, data dir, and explorer support"
```

---

## Task 10: 集成验证 + 修复

确保所有面板正确联动：主题切换生效、语言切换生效、Agent 配置持久化、插件管理正常。

**Files:**
- Modify: `src/App.vue` — 修复 system 主题逻辑
- Modify: `src/plugins/settings/index.ts` — onInit 中应用主题
- All panels — 端到端验证

- [ ] **Step 1: 修复 App.vue 的 system 主题处理**

更新 App.vue 的 theme computed，正确处理 "system" 选项：

```typescript
// src/App.vue script setup 中替换 theme computed
import { useOsTheme } from "naive-ui";

const osTheme = useOsTheme();

const theme = computed(() => {
  const pref = settingsStore.appearance.theme;
  if (pref === "dark") return darkTheme;
  if (pref === "system" && osTheme.value === "dark") return darkTheme;
  return null;
});
```

- [ ] **Step 2: 添加语言切换 watcher**

在 `src/plugins/settings/store.ts` 中添加 i18n locale 同步。在 store 创建后添加：

```typescript
// src/plugins/settings/store.ts 顶部添加 import
import i18n from "@/core/i18n";

// 在 return 之前，load 函数内部的 language 赋值后添加同步
// 同时在 watch 中监听 appearance.language 变化同步 i18n
watch(
  () => appearance.value.language,
  (lang) => {
    i18n.global.locale.value = lang;
  }
);
```

- [ ] **Step 3: 端到端验证**

运行: `npm run tauri dev`

验证清单：
1. 应用启动 → 自动跳转到 /welcome
2. 点击"开始使用" → 跳转到 /settings
3. 外观 Tab：切换主题 → 界面即时响应；切换语言 → 标签文字即时变化
4. Agent Tab：显示 5 个内置 Agent；点击浏览按钮可打开文件夹选择器
5. 插件 Tab：显示设置插件，不可禁用
6. 关于 Tab：显示版本号和数据目录
7. 重启应用 → 设置保持不变

- [ ] **Step 4: 提交**

```bash
git add -A
git commit -m "feat(settings): integrate theme/language switching and verify end-to-end"
```

---

## 完成标志

所有 Task 完成后：
- [ ] 设置插件完全可用（4 个 Tab 全部工作）
- [ ] 主题切换即时生效（亮色/暗色/跟随系统）
- [ ] 语言切换即时生效（中文/英文）
- [ ] Agent 配置持久化（重启后保留）
- [ ] 插件启用/禁用正常（侧边栏响应）
- [ ] 关于页面显示正确信息
