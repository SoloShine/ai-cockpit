# CLAUDE.md

## 项目概述

**AI Cockpit** — 基于 Tauri 2 的通用 AI 管理工具箱，面向泛 AI 用户。采用插件化架构，每个功能模块（Skill 管理、提示词库、开发工具、AI 测试）是独立插件，通过标准接口注册到应用壳。

前端 Vue 3 + TypeScript + Naive UI，后端 Rust。前后端通过 Tauri IPC 通信。

## 产品背景与愿景

### 起源

本项目前身是 `field-skill-manage`（SPM Manager），一个专门管理 AI Agent Skill 包的 Tauri 2 桌面工具（v1.3.0，32 个 IPC 命令，6 个页面）。支持 Claude Code / OpenCode / Codex / Cursor / Windsurf 五种 Agent 的全局和项目级 Skill 安装、更新、校验、卸载、迁移。

### 升级决策

将 SPM Manager 升级为通用 AI 管理工具箱。选择**全新项目**而非在原项目上扩展，原因：

1. 原项目架构为单一功能（Skill CRUD）设计，flat view 结构无法支撑多模块并行
2. 历史代码中的假定（单一功能类型、固定侧边栏）会限制扩展
3. 插件化架构从零设计比改造更干净，避免渐进腐化

### 架构选择：为什么是插件化

评估了三种方案：

| 方案 | 描述 | 结论 |
|------|------|------|
| 渐进式增强 | 在原架构上逐模块加 view/store/commands | 模块增多后侧边栏和路由臃肿 |
| **插件化** | 每个功能模块是标准接口的独立插件 | **选中** — 解耦彻底，支持未来第三方插件 |
| 多窗口 | Tauri 多窗口隔离各模块 | 窗口间通信复杂，UX 碎片化 |

### 目标用户与商业化

- **目标用户**：泛 AI 用户（开发者 + 设计师 + 产品 + 运营等所有使用 AI 工具的人）
- **变现模式**：先产品后商业（当前阶段专注做好产品，暂不考虑付费/账号/云服务）
- **产品定位**：AI 时代的"瑞士军刀"——一个桌面端统一管理所有 AI 相关的配置、工具和资源

### 功能模块规划

| 模块 | 核心能力 | 优先级 | 当前状态 |
|------|---------|--------|---------|
| **Skill 管理** | 多 Agent Skill 安装/更新/卸载/迁移，版本对比，Git 同步 | P0 | 待开发（可参考 field-skill-manage） |
| **提示词库** | 提示词 CRUD、分类、搜索、版本历史、变量模板 | P0 | 待开发 |
| **开发工具集** | JSON/YAML 格式化、正则测试、Base64、时间转换等轻量工具 | P1 | 待开发（纯前端，无 Rust 依赖） |
| **AI 测试** | 待定义（可能是输出评测、Agent 行为测试或模型对比） | P2 | 概念阶段，暂不投入 |
| **设置** | Agent 配置、主题、语言、插件管理 | 核心 | 待开发 |

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

### 插件开发速查

完整规范见 `docs/plugin-development-guide.md`。要点：

- **目录**：`src/plugins/<plugin-id>/index.ts` — 导出 `PluginModule`
- **注册**：在 `src/main.ts` 中 `pluginRegistry.register(module)`
- **路由**：每个 route 的 `meta.pluginId` 必须等于插件 `id`，用懒加载
- **i18n**：插件自管 `i18n/zh-CN.json`，在 `onInit` 中 `mergeLocaleMessage`
- **设置**：导出 `SettingsPanel` 组件 → 自动在设置页成为独立 Tab
- **公共 API**：从 `composables.ts` 导出（如 `useAgentPaths`）
- **跨插件通信**：Pinia store 或公共 composables，禁止直接 import 其他插件的组件
- **Rust 端**：`commands/<id>.rs` + `services/<id>_service.rs`，serde 用 `rename` 对齐 camelCase
- **命名**：ID 用 kebab-case，目录/Rust 模块与 ID 一致

## 开发路线图

**阶段 1 — 基础框架** ✅ 已完成
- [x] Tauri 2 + Vue 3 项目脚手架
- [x] 插件系统核心（types / registry / lifecycle hooks）
- [x] 动态侧边栏 + 路由合并
- [x] App Shell 布局

**阶段 2 — 核心插件**
- [ ] 设置插件（Agent 配置、主题、语言、插件管理）
- [ ] Skill 管理插件（从 field-skill-manage 迁移核心逻辑）
- [ ] 提示词库插件

**阶段 3 — 扩展功能**
- [ ] 开发工具集插件（纯前端工具箱）
- [ ] AI 测试插件（待产品定义）

**阶段 4 — 商业化准备**
- [ ] 用户账号 / 云同步（按需）
- [ ] 团队协作功能（按需）

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
- Skill 发现与版本对比逻辑（services/skill_service.rs，1193 行）
- Git 同步机制（services/git_service.rs — clone --depth 1 / pull --ff-only + 损坏恢复）
- 哈希校验策略（services/hash_service.rs — SHA256 聚合哈希，4 级回退）
- 多 Agent 路径模型（models/config.rs — AgentType 枚举、全局/项目路径模式）
- 版本对比 UI（SkillCompareTable.vue — 批量操作、状态过滤）
- Diff 查看器（SkillDiffViewer.vue — 逐行高亮）
- 跨 Agent 迁移（MigrateDialog.vue — 3 步向导）

## 新会话快速启动

如果你是一个新的 Claude Code 会话，以下是快速进入工作状态的要点：

1. **读 CLAUDE.md**（你正在读）— 包含架构、约定、路线图
2. **看 src/core/plugin/types.ts** — 理解插件接口，这是整个项目的基础契约
3. **看 src/main.ts** — 理解插件注册点和应用启动流程
4. **当前阶段**：阶段 2（核心插件开发），优先做设置插件，然后是 Skill 管理和提示词库
5. **参考代码**：需要实现 Skill 管理时参考 `D:\Project\field-skill-manage`
