# CLAUDE.md

## 项目概述

**AI Cockpit** — 基于 Tauri 2 的通用 AI 管理工具箱，面向泛 AI 用户。采用插件化架构，每个功能模块（Skill 管理、提示词库、开发工具、AI 测试）是独立插件，通过标准接口注册到应用壳。

前端 Vue 3 + TypeScript + Naive UI，后端 Rust。前后端通过 Tauri IPC 通信。

## 常用命令

```bash
npm install                # 安装前端依赖
npm run tauri dev          # 启动开发服务器（Vite :1420 + Rust 热重载）
npm run tauri build        # 生产构建 → src-tauri/target/release/bundle/
npm run build              # 仅前端构建
```

项目未配置测试框架，没有测试命令。

## 插件架构

### 核心概念

每个插件是一个 TypeScript 模块，导出 `CockpitPlugin` + 可选 `PluginHooks`：

```typescript
// src/plugins/<name>/index.ts
export default {
  id: "skills",
  name: "Skill 管理",
  icon: RocketOutline,
  routes: [...],
  navItems: [...],
  order: 10,
} satisfies CockpitPlugin;

export const hooks: PluginHooks = {
  onInit() { ... },
  SettingsPanel: MySettingsPanel,
};
```

### 插件注册流程

1. 插件在 `src/main.ts` 中 import 并调用 `pluginRegistry.register()`
2. Registry 验证依赖、收集路由和导航项
3. Router 从 registry 获取所有路由并注册
4. Sidebar 从 registry 获取导航项并渲染菜单
5. 路由切换时触发 `onActivate`/`onDeactivate` 生命周期钩子

### 插件接口定义

定义在 `src/core/plugin/types.ts`：

- **CockpitPlugin** — id, name, icon, routes, navItems, order, dependsOn
- **PluginHooks** — onInit, onActivate, onDeactivate, SettingsPanel
- **NavItem** — routeName, label, icon, children

### 后端命令注册约定

每个插件如果需要 Rust 后端能力，在 `src-tauri/src/commands/` 下创建同名模块（如 `skills.rs`），在 `lib.rs` 的 `invoke_handler` 中注册。服务层放 `services/` 下。

## 当前插件规划

| 插件 | ID | 优先级 | 状态 |
|------|-----|--------|------|
| Skill 管理 | `skills` | P0 | 待开发 |
| 提示词库 | `prompts` | P0 | 待开发 |
| 开发工具集 | `devtools` | P1 | 待开发 |
| AI 测试 | `testing` | P2 | 概念阶段 |
| 设置 | `settings` | 核心 | 待开发 |

## 前端结构

```
src/
├── core/                  # 应用壳（不依赖任何插件）
│   ├── plugin/            # 插件系统（registry, types）
│   ├── layout/            # AppLayout, Sidebar
│   ├── theme/             # 主题系统
│   └── i18n/              # 国际化
├── plugins/               # 各功能插件（每个子目录一个）
│   ├── skills/
│   ├── prompts/
│   └── devtools/
├── stores/                # Pinia stores（仅 plugin store 是核心的）
├── views/                 # 仅 WelcomeView 和 SettingsView 属于核心
├── types/                 # 共享类型
├── router/                # 路由（核心 + 插件动态注册）
└── main.ts                # 入口（插件注册点）
```

### 路由规则

- 核心路由：`/welcome`（首页）、`/settings`（全局设置）
- 插件路由：每个插件定义自己的路由，meta 中标注 `pluginId`
- 路由守卫根据 `pluginId` 追踪活跃插件

## Rust 后端结构

```
src-tauri/src/
├── main.rs
├── lib.rs                 # Tauri Builder，注册所有命令
├── commands/              # IPC 命令处理器
│   ├── mod.rs
│   └── core.rs            # 核心命令（get_app_version）
├── services/              # 业务逻辑（按插件分模块）
└── models/                # 共享数据模型
```

### 新增插件的 Rust 端步骤

1. `commands/` 下新建 `<plugin>.rs`
2. `services/` 下新建对应 service
3. `lib.rs` 的 `invoke_handler` 中添加命令

## 技术栈

| 层 | 技术 |
|----|------|
| 前端框架 | Vue 3.5 + TypeScript 5 |
| UI 库 | Naive UI 2 |
| 状态管理 | Pinia 3 |
| 路由 | Vue Router 4 |
| 国际化 | vue-i18n 11 |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust |
| 构建 | Vite 6 |

## 关键约定

- 路径别名：`@` → `src/`
- 插件之间通过 Pinia store 或事件总线通信，禁止直接 import 另一个插件的内部组件
- 所有 Rust 命令返回 `Result<T, String>`
- i18n key 格式：`<pluginId>.<section>.<key>`（如 `skills.compare.outdated`）
- 每个插件独立管理自己的 i18n 资源

## 参考项目

前身项目 `field-skill-manage`（SPM Manager）位于 `D:\Project\field-skill-manage`，可作为 Skill 管理插件实现的参考。主要参考：
- Skill 发现与版本对比逻辑（services/skill_service.rs）
- Git 同步机制（services/git_service.rs）
- 哈希校验策略（services/hash_service.rs）
- 多 Agent 路径模型（models/config.rs）
