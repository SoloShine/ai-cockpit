# 设置插件设计

## 概述

AI Cockpit 的设置插件 —— 管理外观主题、Agent 路径、插件生命周期和应用维护信息。作为标准 `CockpitPlugin` 注册，无特权。在 `onInit` 时加载主题/语言并全局应用。

## 架构

设置是常规插件（`id: "settings"`），遵循与其他插件相同的 `CockpitPlugin` 契约。注册 `/settings` 路由并提供导航项。侧边栏已有固定的设置入口，指向同一路径。

其他插件通过 `PluginHooks.SettingsPanel` 组件贡献自己的设置面板 —— 设置插件在"插件管理"Tab 中发现并渲染这些面板。

## 数据模型

```typescript
interface AppSettings {
  appearance: {
    theme: "light" | "dark" | "system";
    language: "zh-CN" | "en-US";
    fontSize: number; // 12-20，默认 14
  };
  agents: AgentConfig[];
  plugins: {
    disabledIds: string[];
    order: string[];
  };
  _meta: {
    version: number;
    updatedAt: string;
  };
}

interface AgentConfig {
  id: string;        // 唯一标识，如 "claude-code"
  name: string;      // 显示名称
  type: string;      // Agent 类型标识符
  basePath: string;  // 配置根路径
  enabled: boolean;
  isCustom: boolean; // 用户自定义 vs 内置
}
```

### 默认 Agent

首次启动时预填充以下内置 Agent：

| ID            | 名称        | 默认路径                |
| ------------- | ----------- | ----------------------- |
| `claude-code` | Claude Code | `~/.claude/commands`    |
| `cursor`      | Cursor      | 平台相关                |
| `windsurf`    | Windsurf    | 平台相关                |
| `opencode`    | OpenCode    | `~/.opencode/commands`  |
| `codex`       | Codex       | `~/.codex/commands`     |

用户可在此五种之外添加自定义 Agent。内置 Agent 只能禁用，不能删除。

## 存储层

两层持久化：

1. **Pinia `settingsStore`** —— 响应式状态，组件直接使用
2. **Rust `settings_commands.rs`** —— `load_settings` / `save_settings` 命令，读写 `~/.ai-cockpit/settings.json`

### 云同步接口（预留）

```typescript
interface SettingsRepository {
  load(): Promise<AppSettings>;
  save(settings: AppSettings): Promise<void>;
}
```

当前实现：`LocalFileRepository`，写入应用数据目录。未来可替换为 `CloudRepository`，消费者无需修改。

### 应用流程

1. 插件 `onInit` 调用 `settingsStore.load()` → 调用 Rust `load_settings`
2. Store 通过全局 ref 将主题应用到 Naive UI `NConfigProvider`
3. Store 将语言应用到 `vue-i18n` locale
4. 任何设置变更触发自动保存（防抖 500ms）

## UI 结构

### 布局

```
┌──────────────┬─────────────────────────────┐
│  NTabs       │  内容区                      │
│  (竖向标签)   │                             │
│              │  (动态面板组件)               │
│  外观与语言   │                             │
│  Agent 配置  │                             │
│  插件管理     │                             │
│  关于        │                             │
└──────────────┴─────────────────────────────┘
```

### Tab 1：外观与语言

- 主题选择器：3 个单选卡片（亮色 / 暗色 / 跟随系统），带预览图标
- 语言下拉框：简体中文 / English
- 字体大小滑块：12-20，步进 1，默认 14
- 变更即时生效，无需保存按钮

### Tab 2：Agent 配置

- Agent 列表以 `NCollapse` 或 `NCard` 网格形式展示
- 每个 Agent 卡片包含：
  - 名称 + 类型标签
  - 基础路径输入框（文本框 + 文件夹浏览按钮，通过 Tauri dialog）
  - 启用/禁用开关
  - 删除按钮（仅自定义 Agent 可见）
- "添加自定义 Agent" 按钮打开对话框：
  - 名称（必填）
  - 类型标识符（必填）
  - 基础路径（必填，带文件夹浏览）
- 校验：保存时路径必须存在

### Tab 3：插件管理

- 插件列表来自 `pluginRegistry.getAll()`
- 每个插件行包含：
  - 图标 + 名称 + 描述
  - 启用/禁用开关（写入 `settings.plugins.disabledIds`）
  - 展开箭头显示详情面板
- 详情面板：
  - 该插件注册的路由列表
  - 插件专属设置面板（来自 `PluginHooks.SettingsPanel`），或显示"暂无设置"
- 拖拽手柄用于排序（写入 `settings.plugins.order`）
- 设置插件本身不可被禁用

### Tab 4：关于与维护

- 应用名称 + 版本号（通过 Tauri `get_app_version` 命令获取）
- 数据目录路径 + "在资源管理器中打开"按钮
- "检查更新"按钮（调用 Tauri updater，当前为占位符）
- "打开日志"按钮（在资源管理器中打开日志目录）
- 技术栈信息（Tauri 版本、Vue 版本）

## 文件结构

```
src/plugins/settings/
├── index.ts                    # CockpitPlugin 导出 + hooks
├── views/
│   └── SettingsView.vue        # Tab 布局外壳
├── panels/
│   ├── AppearancePanel.vue     # 主题、语言、字体大小
│   ├── AgentPanel.vue          # Agent 列表 + 增删改
│   ├── PluginPanel.vue         # 插件列表 + 启用禁用 + 排序
│   └── AboutPanel.vue          # 版本、数据目录、日志
├── components/
│   ├── ThemeSelector.vue       # 主题单选卡片
│   ├── AgentCard.vue           # 单个 Agent 配置卡片
│   ├── AddAgentDialog.vue      # 添加自定义 Agent 对话框
│   └── PluginDetail.vue        # 展开的插件详情 + 设置
├── store.ts                    # Pinia settingsStore
├── types.ts                    # AppSettings、AgentConfig 接口
└── i18n/
    ├── zh-CN.json              # settings.appearance.*、settings.agents.* 等
    └── en-US.json

src-tauri/src/
├── commands/
│   └── settings.rs             # load_settings、save_settings
├── services/
│   └── settings_service.rs     # 文件读写、默认设置生成
```

## 插件注册中心集成

设置 store 监听插件注册中心：

```typescript
// 加载时，将注册中心的插件与存储的排序/禁用状态合并
const registeredPlugins = pluginRegistry.getAll();
const { disabledIds, order } = settings.plugins;

// 按存储的顺序排序，未记录的使用插件自身 order
const sorted = sortBy(registeredPlugins, order);
// 过滤掉已禁用的插件
const enabled = sorted.filter(p => !disabledIds.includes(p.id));
```

禁用插件时：

1. 将 id 添加到 `disabledIds`
2. 插件注册中心移除其路由和导航项
3. 如果用户当前在被禁用插件的路由上，重定向到 `/welcome`

插件排序变更时：

1. 更新 `order` 数组
2. 侧边栏按新顺序重新渲染导航项

## 依赖

- `@tauri-apps/plugin-dialog` —— Agent 路径的文件夹选择器
- `@tauri-apps/plugin-fs` —— 已在 package.json 中
- `pinia` —— settingsStore
- `naive-ui` —— 所有 UI 组件（NTabs、NCard、NSwitch、NSlider、NCollapse 等）
- `vue-i18n` —— 设置面板国际化

无需新增 package.json 中不存在的依赖。

## 不在 v1 范围内

- 云同步实现（仅预留接口）
- 导入/导出设置
- 插件权限管理
- 自动更新功能（按钮为占位符）
