# E2E Testing Framework Design

## 概述

为 ai-cockpit 添加基于 WDIO 9 + tauri-driver 的 E2E 测试框架，采用独立 `e2e/` 目录结构（与 scene-todo 一致），包含基础配置和示例测试。

## 方案选择

| 方案 | 描述 | 结论 |
|------|------|------|
| **WDIO 9 + tauri-driver** | Tauri 官方推荐的 WebDriver 桥接方案，与参考项目 scene-todo 一致 | **选中** |
| Playwright + tauri-driver | API 更现代但与 tauri-driver 集成不成熟 | 未选 |
| 纯前端测试（Cypress/Playwright） | 无法测试 Tauri IPC 和原生窗口行为 | 不适用 |

## 目录结构

```
e2e/
├── package.json       # 独立依赖（WDIO 9、tsx、TypeScript）
├── wdio.conf.ts       # 测试配置
└── specs/
    └── welcome.spec.ts  # 示例测试
```

## 依赖

| 包 | 版本 | 用途 |
|----|------|------|
| @wdio/cli | ^9 | WDIO CLI |
| @wdio/local-runner | ^9 | 本地测试运行器 |
| @wdio/mocha-framework | ^9 | Mocha BDD 测试框架 |
| @wdio/spec-reporter | ^9 | 测试报告 |
| tsx | ^4 | 直接运行 TS 配置 |
| typescript | ^5 | TypeScript 支持 |
| @types/node | ^22 | Node.js 类型 |

## wdio.conf.ts 配置要点

- **host/port**：`127.0.0.1:4444`（tauri-driver 默认）
- **capabilities**：`tauri:options` 指向 `src-tauri/target/debug/AI Cockpit`（取自 tauri.conf.json productName）
- **framework**：mocha + bdd，timeout 60s
- **onPrepare**：自动构建 debug 二进制（若不存在）
- **beforeSession**：启动 tauri-driver 进程
- **afterSession**：清理 tauri-driver 进程
- **跨平台**：检测 Windows/Unix，自动添加 `.exe` 后缀

## 前置条件

- 全局安装 tauri-driver：`cargo install tauri-driver --locked`

## 示例测试

### welcome.spec.ts

1. **应用启动验证** — 窗口标题为 "AI Cockpit"
2. **欢迎页渲染** — 验证欢迎页关键元素存在

## npm scripts

```json
{
  "test": "wdio run wdio.conf.ts"
}
```

后续按模块扩展：`test:settings`、`test:skills` 等。

## 测试编写约定

- 使用 `data-testid` 属性作为选择器（需在组件中添加）
- 测试文件命名：`<module>.spec.ts`
- 辅助函数放在各 spec 文件内（当前规模不需要单独的 page objects）
- 测试数据用 `Date.now().toString(36)` 生成唯一 ID 避免冲突

## 与 scene-todo 的差异

| 项目 | scene-todo | ai-cockpit |
|------|-----------|------------|
| 前端框架 | React 18 | Vue 3 + Naive UI |
| 二进制名 | scene-todo | AI Cockpit |
| edgedriver | 有 | 不需要（tauri-driver 足够） |
| 测试覆盖 | 10 个模块，完整覆盖 | 1 个示例，后续扩展 |
