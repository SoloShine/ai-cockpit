# AI Cockpit

基于 Tauri 2 的通用 AI 管理工具箱，采用插件化架构。

## 功能模块

| 模块 | 说明 | 状态 |
|------|------|------|
| Skill 管理 | 多 Agent Skill 安装/更新/卸载/迁移 | 规划中 |
| 提示词库 | 提示词 CRUD、分类、搜索 | 规划中 |
| 开发工具集 | JSON/YAML 格式化、正则测试等轻量工具 | 规划中 |
| AI 测试 | 待定义 | 概念阶段 |
| 设置 | Agent 配置、主题、语言、插件管理 | 开发中 |

## 技术栈

- **前端**：Vue 3 + TypeScript + Naive UI + Pinia + Vue Router + vue-i18n
- **桌面**：Tauri 2
- **后端**：Rust
- **构建**：Vite 6

## 开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 生产构建
npm run tauri build
```

## E2E 测试

基于 WDIO 9 + tauri-driver，测试文件位于 `e2e/` 目录。

```bash
# 前置条件（首次）
cargo install tauri-driver --locked

# 安装 e2e 依赖（首次）
cd e2e && npm install

# 运行全部测试
cd e2e && npm test

# 运行单个测试
cd e2e && npx wdio run wdio.conf.ts --spec ./specs/welcome.spec.ts
```

### 编写新测试

1. 在被测组件中添加 `data-testid` 属性
2. 在 `e2e/specs/` 下创建 `<module>.spec.ts`
3. 在 `e2e/package.json` 中添加 `test:<module>` 脚本

### 测试约定

- 使用 `data-testid` 作为选择器，不依赖 CSS 类名或 DOM 结构
- 测试文件命名：`<module>.spec.ts`
- 测试数据用 `Date.now().toString(36)` 生成唯一 ID

## 项目结构

```
src/                      # 前端源码
├── core/                 # 应用壳（插件系统、布局、主题、i18n）
├── plugins/              # 功能插件（每个子目录一个）
├── stores/               # Pinia stores
├── views/                # 核心页面
├── router/               # 路由
└── main.ts               # 入口
src-tauri/                # Rust 后端
├── src/commands/         # IPC 命令处理器
├── src/services/         # 业务逻辑
└── src/models/           # 数据模型
e2e/                      # E2E 测试（独立 package.json）
├── wdio.conf.ts          # WDIO 配置
└── specs/                # 测试文件
```

## License

Private
