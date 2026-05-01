# E2E Testing Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add WDIO 9 + tauri-driver E2E testing framework with a working example test.

**Architecture:** Separate `e2e/` directory with its own `package.json` and `wdio.conf.ts`. The WDIO runner builds the Tauri debug binary if needed, starts tauri-driver, runs tests against the real app, then cleans up. Mirrors the pattern from scene-todo.

**Tech Stack:** WebDriverIO 9, Mocha BDD, tauri-driver, TypeScript

---

## File Structure

```
e2e/
├── package.json           # E2E dependencies (WDIO 9, tsx, TypeScript)
├── wdio.conf.ts           # Test runner config (build, driver lifecycle, capabilities)
└── specs/
    └── welcome.spec.ts    # Example test: app launch + welcome page
```

No existing files are modified — the entire change is additive under `e2e/`.

**Note on binary name:** Tauri produces the debug binary from the Cargo.toml `[package] name` field (`ai-cockpit`), not the tauri.conf.json `productName`. The binary path is `src-tauri/target/debug/ai-cockpit` (or `ai-cockpit.exe` on Windows).

---

### Task 1: Create e2e/package.json

**Files:**
- Create: `e2e/package.json`

- [ ] **Step 1: Create e2e directory and package.json**

```bash
mkdir -p e2e/specs
```

```json
{
  "name": "ai-cockpit-e2e",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "wdio run wdio.conf.ts"
  },
  "dependencies": {
    "@types/node": "^22.15.0",
    "@wdio/cli": "^9.12.0",
    "@wdio/local-runner": "^9.12.0",
    "@wdio/mocha-framework": "^9.12.0",
    "@wdio/spec-reporter": "^9.12.0",
    "tsx": "^4.19.0",
    "typescript": "^5.8.0"
  }
}
```

- [ ] **Step 2: Install dependencies**

Run: `cd e2e && npm install`
Expected: `node_modules` created, `package-lock.json` generated.

- [ ] **Step 3: Commit**

```bash
git add e2e/package.json e2e/package-lock.json
git commit -m "chore(e2e): add e2e package.json with WDIO 9 dependencies"
```

---

### Task 2: Create wdio.conf.ts

**Files:**
- Create: `e2e/wdio.conf.ts`

This config mirrors scene-todo's setup adapted for ai-cockpit's binary name (`ai-cockpit` from Cargo.toml). Key differences from scene-todo: no `edgedriver` dependency.

- [ ] **Step 1: Create wdio.conf.ts**

```typescript
import os from "os";
import path from "path";
import fs from "fs";
import { spawn, spawnSync } from "child_process";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const isWindows = os.platform() === "win32";

// Binary name comes from Cargo.toml [package] name, not tauri.conf.json productName
const application = path.resolve(
  __dirname,
  "..",
  "src-tauri",
  "target",
  "debug",
  isWindows ? "ai-cockpit.exe" : "ai-cockpit"
);

let tauriDriver: ReturnType<typeof spawn> | null = null;
let exit = false;

function closeTauriDriver() {
  exit = true;
  if (tauriDriver) {
    tauriDriver.kill();
    tauriDriver = null;
  }
}

function onShutdown(fn: () => void) {
  const cleanup = () => {
    try { fn(); } finally { process.exit(); }
  };
  process.on("exit", cleanup);
  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);
  if (isWindows) process.on("SIGBREAK", cleanup);
  else process.on("SIGHUP", cleanup);
}

onShutdown(() => closeTauriDriver());

export const config = {
  host: "127.0.0.1",
  port: 4444,
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  onPrepare() {
    if (fs.existsSync(application)) {
      console.log("Debug binary exists, skipping build.");
      return;
    }
    console.log("Building Tauri debug binary...");
    const result = spawnSync(
      "npm",
      ["run", "tauri", "build", "--", "--debug", "--no-bundle"],
      {
        cwd: path.resolve(__dirname, ".."),
        stdio: "inherit",
        shell: true,
      }
    );
    if (result.status !== 0) {
      throw new Error("Tauri build failed");
    }
    console.log("Build complete.");
  },

  beforeSession() {
    const driverPath = path.resolve(
      os.homedir(),
      ".cargo",
      "bin",
      isWindows ? "tauri-driver.exe" : "tauri-driver"
    );
    console.log("Starting tauri-driver:", driverPath);
    tauriDriver = spawn(driverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });
    tauriDriver.on("error", (error: Error) => {
      console.error("tauri-driver error:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code: number | null) => {
      if (!exit) {
        console.error("tauri-driver exited with code:", code);
        process.exit(1);
      }
    });
  },

  afterSession() {
    closeTauriDriver();
  },
};
```

- [ ] **Step 2: Commit**

```bash
git add e2e/wdio.conf.ts
git commit -m "chore(e2e): add WDIO config with tauri-driver lifecycle management"
```

---

### Task 3: Add data-testid to WelcomeView

**Files:**
- Modify: `src/views/WelcomeView.vue`

The welcome page currently has no `data-testid` attributes. Add them so the E2E test can reliably select elements without depending on CSS classes or DOM structure.

- [ ] **Step 1: Add data-testid attributes to WelcomeView**

Current `WelcomeView.vue` template:
```html
<template>
  <div style="display: flex; align-items: center; justify-content: center; height: 80%">
    <NResult status="info" title="AI Cockpit" description="通用 AI 管理工具箱">
      <template #footer>
        <NButton @click="router.push({ name: 'settings' })">开始使用</NButton>
      </template>
    </NResult>
  </div>
</template>
```

Modified template — add `data-testid` to the wrapper div and the button:
```html
<template>
  <div data-testid="welcome-page" style="display: flex; align-items: center; justify-content: center; height: 80%">
    <NResult status="info" title="AI Cockpit" description="通用 AI 管理工具箱">
      <template #footer>
        <NButton data-testid="welcome-start-btn" @click="router.push({ name: 'settings' })">开始使用</NButton>
      </template>
    </NResult>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/WelcomeView.vue
git commit -m "feat: add data-testid to WelcomeView for E2E testing"
```

---

### Task 4: Create example E2E test

**Files:**
- Create: `e2e/specs/welcome.spec.ts`

- [ ] **Step 1: Create welcome.spec.ts**

```typescript
describe("Welcome Page", () => {
  it("should display the app window", async () => {
    const title = await browser.getTitle();
    expect(title).toBe("AI Cockpit");
  });

  it("should render the welcome page", async () => {
    const welcomePage = await $("[data-testid='welcome-page']");
    await welcomePage.waitForExist({ timeout: 10000 });
    expect(await welcomePage.isExisting()).toBe(true);
  });

  it("should have a start button that navigates to settings", async () => {
    const startBtn = await $("[data-testid='welcome-start-btn']");
    await startBtn.waitForClickable({ timeout: 5000 });
    await startBtn.click();
    await browser.pause(500);
    // Verify navigation away from welcome page
    const url = await browser.getUrl();
    expect(url).toContain("/settings");
  });
});
```

- [ ] **Step 2: Verify tauri-driver is installed**

Run: `tauri-driver --version`
Expected: prints version number. If not found, run: `cargo install tauri-driver --locked`

- [ ] **Step 3: Build the debug binary (first time only)**

Run: `cd /d/Project/ai-cockpit/.claude/worktrees/lucid-solomon-184df1 && npm run tauri build -- --debug --no-bundle`
Expected: Binary at `src-tauri/target/debug/ai-cockpit.exe`

- [ ] **Step 4: Run the E2E test**

Run: `cd e2e && npm test`
Expected: All 3 tests pass with spec reporter output.

- [ ] **Step 5: Commit**

```bash
git add e2e/specs/welcome.spec.ts
git commit -m "test(e2e): add welcome page E2E tests"
```

---

### Task 5: Add .gitignore entry for e2e

**Files:**
- Modify: `.gitignore` (add `e2e/node_modules/` if not already covered)

- [ ] **Step 1: Check .gitignore and update if needed**

Check if `node_modules` is already gitignored at root level. If `e2e/node_modules/` is already covered by a root `node_modules` entry, skip. Otherwise add:

```
e2e/node_modules/
```

- [ ] **Step 2: Commit (if changed)**

```bash
git add .gitignore
git commit -m "chore: gitignore e2e node_modules"
```

---

## Self-Review

**Spec coverage:**
- Directory structure (e2e/ with package.json, wdio.conf.ts, specs/) → Task 1, 2, 4
- Dependencies (WDIO 9, tsx, typescript, @types/node) → Task 1
- wdio.conf.ts config (host/port, capabilities, build, driver lifecycle) → Task 2
- Example test (app launch, welcome page) → Task 3 + 4
- npm scripts → Task 1
- data-testid convention → Task 3

**Placeholder scan:** No TBD/TODO/placeholder patterns found.

**Type consistency:** Binary name `ai-cockpit` used consistently across wdio.conf.ts (matching Cargo.toml `[package] name`).
