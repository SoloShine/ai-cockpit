# AI Cockpit 插件开发规范

## 概述

AI Cockpit 采用插件化架构，每个功能模块（Skill 管理、提示词库、开发工具等）是独立插件，通过标准接口注册到应用壳。

插件是 TypeScript 模块，导出 `CockpitPlugin` 清单 + 可选 `PluginHooks` 生命周期钩子。需要 Rust 后端能力的插件，同时在 `src-tauri/` 下创建对应的命令和服务模块。

## 最小可用插件

```typescript
// src/plugins/example/index.ts
import { RocketOutline } from "@vicons/ionicons5";
import type { CockpitPlugin, PluginModule } from "@/core/plugin";

const plugin: CockpitPlugin = {
  id: "example",
  name: "示例插件",
  icon: RocketOutline,
  routes: [
    {
      path: "/example",
      name: "example",
      component: () => import("./views/ExampleView.vue"),
      meta: { pluginId: "example" },
    },
  ],
  navItems: [
    { routeName: "example", label: "示例", icon: RocketOutline },
  ],
};

const module: PluginModule = {
  default: plugin,
};

export default module;
```

注册到 `src/main.ts`：

```typescript
import exampleModule from "./plugins/example";
pluginRegistry.register(exampleModule);
```

## 目录结构

```
src/plugins/<plugin-id>/
├── index.ts              # 必须 — 插件注册入口
├── types.ts              # 插件内部类型
├── store.ts              # Pinia store（如需要）
├── composables.ts        # 公共 API 导出（供其他插件调用）
├── views/                # 路由页面
│   └── MainView.vue
├── panels/               # 设置面板子组件
├── components/           # 可复用 UI 组件
└── i18n/                 # 国际化资源
    ├── zh-CN.json
    └── en-US.json
```

## 插件清单（CockpitPlugin）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | `string` | 是 | 唯一标识，kebab-case（如 `skill-manage`） |
| `name` | `string` | 是 | 侧边栏显示名，支持 i18n key |
| `description` | `string` | 否 | 简短描述，显示在插件管理页 |
| `icon` | `Component \| string` | 是 | 侧边栏图标，推荐 `@vicons/ionicons5` |
| `routes` | `RouteRecordRaw[]` | 是 | 插件的路由，meta 中必须标注 `pluginId` |
| `navItems` | `NavItem[]` | 是 | 侧边栏导航项 |
| `order` | `number` | 否 | 排序权重，越小越靠前，默认 100 |
| `dependsOn` | `string[]` | 否 | 依赖的插件 ID 列表 |

### NavItem

```typescript
interface NavItem {
  routeName: string;          // 路由 name
  label: string;              // 显示文本，支持 i18n key
  icon?: Component | string;  // 图标
  children?: NavItem[];       // 子导航（嵌套菜单）
}
```

单个 navItem → 直接显示为一行菜单项。
多个 navItem → 自动折叠为带子菜单的分组。

### 路由规则

- 每个 route 的 `meta.pluginId` 必须等于插件 `id`
- 路由路径建议以 `/<plugin-id>` 开头
- 使用 `() => import(...)` 懒加载

## 生命周期钩子（PluginHooks）

```typescript
interface PluginHooks {
  onInit?: () => void | Promise<void>;
  onActivate?: () => void | Promise<void>;
  onDeactivate?: () => void | Promise<void>;
  SettingsPanel?: Component;
}
```

| 钩子 | 时机 | 用途 |
|------|------|------|
| `onInit` | 应用启动时，所有插件注册后、挂载前 | 合并 i18n 消息、加载持久化配置、初始化 store |
| `onActivate` | 用户首次导航到该插件路由时 | 懒加载数据、启动定时器 |
| `onDeactivate` | 用户离开该插件路由时 | 清理定时器、保存草稿 |
| `SettingsPanel` | 设置页自动渲染 | 贡献插件专属设置面板 |

### onInit 执行顺序

1. Pinia 和 i18n 先安装
2. 所有插件 `register()`
3. 动态添加路由
4. 依次调用所有插件的 `onInit`（按 order 排序）
5. 挂载应用

**注意**：`onInit` 是 async 的，应用会等待所有 `onInit` 完成后才挂载。

## 国际化（i18n）

每个插件管理自己的 i18n 资源。

### 资源文件

```json
// src/plugins/<id>/i18n/zh-CN.json
{
  "<pluginId>": {
    "section": {
      "key": "中文文本"
    }
  }
}
```

### key 格式

`<pluginId>.<section>.<key>`，如 `skills.compare.outdated`。

### 合并方式

在 `onInit` 中合并到全局 i18n：

```typescript
import zhCN from "./i18n/zh-CN.json";
import enUS from "./i18n/en-US.json";
import i18n from "@/core/i18n";

const hooks: PluginHooks = {
  onInit() {
    i18n.global.mergeLocaleMessage("zh-CN", zhCN);
    i18n.global.mergeLocaleMessage("en-US", enUS);
  },
};
```

## 设置面板集成

导出 `SettingsPanel` 组件，会自动在设置页面作为独立 Tab 显示：

```typescript
import MySettingsPanel from "./panels/SettingsPanel.vue";

const hooks: PluginHooks = {
  SettingsPanel: MySettingsPanel,
};
```

Tab 顺序：外观 → Agent → [插件设置 Tab...] → 插件管理 → 关于。

## Rust 后端集成

需要 Rust 后端能力的插件遵循以下步骤：

### 1. 创建命令模块

```rust
// src-tauri/src/commands/<plugin-id>.rs
#[tauri::command]
pub fn my_command() -> Result<String, String> {
    // 调用 service 层
}
```

### 2. 创建服务模块

```rust
// src-tauri/src/services/<plugin-id>_service.rs
pub fn my_logic() -> Result<String, String> {
    // 业务逻辑
}
```

### 3. 注册模块

```rust
// src-tauri/src/commands/mod.rs
pub mod core;
pub mod settings;
pub mod <plugin-id>;

// src-tauri/src/services/mod.rs
pub mod settings_service;
pub mod <plugin-id>_service;
```

### 4. 注册命令

```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    commands::core::get_app_version,
    commands::<plugin-id>::my_command,
])
```

### Serde 命名约定

Rust 用 snake_case，TypeScript 用 camelCase。使用 `#[serde(rename = "...")]` 对齐：

```rust
#[derive(Serialize, Deserialize)]
pub struct MyData {
    #[serde(rename = "myField")]
    pub my_field: String,
}
```

## 插件间通信

### 允许的方式

- **Pinia store** — 直接 import 另一个插件的 store
- **公共 composables** — 从 `composables.ts` 导出的函数
- **事件总线** — 待实现

### 禁止的方式

- 直接 import 另一个插件的 Vue 组件
- 直接操作另一个插件的内部状态

### 使用设置插件的公共 API

```typescript
import { useAgentPaths, useAppAppearance } from "@/plugins/settings/composables";

// 获取已启用的 Agent 列表
const { enabledAgents, getAgentById } = useAgentPaths();

// 获取外观配置
const { theme, language } = useAppAppearance();
```

### 查询插件状态

```typescript
import { usePluginEnabled } from "@/plugins/settings/composables";

const isSkillsEnabled = usePluginEnabled("skills");
```

## 命名约定

| 项目 | 规范 | 示例 |
|------|------|------|
| 插件 ID | kebab-case | `skill-manage` |
| 插件目录 | 与 ID 一致 | `src/plugins/skill-manage/` |
| 路由路径 | `/<plugin-id>` | `/skill-manage` |
| 路由 name | 与 ID 一致 | `skill-manage` |
| i18n key | `<pluginId>.<section>.<key>` | `skills.compare.outdated` |
| Rust 命令模块 | `<plugin-id>.rs` | `skill_manage.rs` |
| Rust 服务模块 | `<plugin-id>_service.rs` | `skill_manage_service.rs` |
| Store 文件 | `store.ts` | `src/plugins/skills/store.ts` |
| 公共 API | `composables.ts` | `src/plugins/skills/composables.ts` |

## 完整示例

参考 `src/plugins/settings/` 目录，它是第一个完整插件实现，包含：
- 4 个 Tab 面板
- Rust 后端命令（load/save settings, get_data_dir, open_in_explorer）
- i18n 资源合并
- onInit 生命周期
- 公共 composables（useAgentPaths, useAppAppearance, usePluginEnabled）
- SettingsPanel hook（自身未使用，但可供其他插件参考）
