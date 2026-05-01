// src/plugins/settings/store.ts
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, AppearanceSettings, AgentConfig, PluginSettings, RepoConfig } from "./types";
import i18n from "@/core/i18n";

const DEFAULT_APPEARANCE: AppearanceSettings = {
  theme: "system",
  language: "zh-CN",
  fontSize: 14,
};

const DEFAULT_AGENTS: AgentConfig[] = [
  { id: "claude-code", name: "Claude Code", type: "claude-code", globalPath: ".claude", projectPath: ".claude", enabled: true, isCustom: false },
  { id: "cursor", name: "Cursor", type: "cursor", globalPath: ".cursor", projectPath: ".cursor", enabled: true, isCustom: false },
  { id: "windsurf", name: "Windsurf", type: "windsurf", globalPath: ".codeium/windsurf", projectPath: ".windsurf", enabled: true, isCustom: false },
  { id: "opencode", name: "OpenCode", type: "opencode", globalPath: ".config/opencode", projectPath: ".opencode", enabled: true, isCustom: false },
  { id: "codex", name: "OpenAI Codex", type: "codex", globalPath: ".codex", projectPath: ".codex", enabled: true, isCustom: false },
  { id: "cline", name: "Cline", type: "cline", globalPath: "Documents/Cline/Rules", projectPath: ".clinerules", enabled: true, isCustom: false },
  { id: "augment", name: "Augment", type: "augment", globalPath: ".augment", projectPath: ".augment", enabled: true, isCustom: false },
  { id: "aider", name: "Aider", type: "aider", globalPath: ".aider", projectPath: ".aider", enabled: true, isCustom: false },
  { id: "copilot", name: "GitHub Copilot", type: "copilot", globalPath: "github/copilot", projectPath: ".github", enabled: true, isCustom: false },
  { id: "trae", name: "Trae", type: "trae", globalPath: ".trae", projectPath: ".trae", enabled: true, isCustom: false },
];

const DEFAULT_PLUGINS: PluginSettings = {
  disabledIds: [],
  order: [],
};

const DEFAULT_REPOS: RepoConfig[] = [];

export const useSettingsStore = defineStore("settings", () => {
  const loaded = ref(false);
  const appearance = ref<AppearanceSettings>({ ...DEFAULT_APPEARANCE });
  const agents = ref<AgentConfig[]>([...DEFAULT_AGENTS]);
  const repos = ref<RepoConfig[]>([...DEFAULT_REPOS]);
  const plugins = ref<PluginSettings>({ ...DEFAULT_PLUGINS });

  // Git Sync state
  const syncResults = ref<import('@/plugins/skills/types').SyncResult[]>([]);
  const syncing = ref(false);

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  async function load() {
    try {
      const settings = await invoke<AppSettings>("load_settings");
      appearance.value = settings.appearance;
      agents.value = settings.agents;
      repos.value = settings.repos ?? [];
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
      repos: repos.value,
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

  watch([appearance, agents, repos, plugins], () => scheduleSave(), { deep: true });

  // 同步语言设置到 i18n
  watch(
    () => appearance.value.language,
    (lang) => {
      i18n.global.locale.value = lang;
    }
  );

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

  function addRepo(repo: RepoConfig) {
    repos.value.push(repo);
  }

  function removeRepo(id: string) {
    repos.value = repos.value.filter((r) => r.id !== id);
  }

  function updateRepo(id: string, updates: Partial<RepoConfig>) {
    const idx = repos.value.findIndex((r) => r.id === id);
    if (idx !== -1) {
      repos.value[idx] = { ...repos.value[idx], ...updates };
    }
  }

  // --- Git Sync ---

  async function syncAllRepos(): Promise<import('@/plugins/skills/types').SyncResult[]> {
    syncing.value = true;
    try {
      const results = await invoke<import('@/plugins/skills/types').SyncResult[]>("sync_all_repos", {
        repos: repos.value,
      });
      syncResults.value = results;

      // Update cache paths for repos that don't have one yet
      for (const result of results) {
        if (result.success) {
          const repo = repos.value.find((r) => r.id === result.repoId);
          if (repo && !repo.cachePath) {
            updateRepo(repo.id, {
              cachePath: `repos/${repo.id}`,
            });
          }
        }
      }
      return results;
    } finally {
      syncing.value = false;
    }
  }

  async function getRemoteSkills(repoId: string): Promise<import('@/plugins/skills/types').RemoteSkillInfo[]> {
    const repo = repos.value.find((r) => r.id === repoId);
    if (!repo) return [];
    return invoke<import('@/plugins/skills/types').RemoteSkillInfo[]>("get_remote_skills", {
      repoId,
      cachePath: repo.cachePath,
    });
  }

  // --- Config Portability ---

  async function exportConfig(): Promise<string> {
    const settings: AppSettings = {
      appearance: appearance.value,
      agents: agents.value,
      repos: repos.value,
      plugins: plugins.value,
      _meta: { version: 1, updatedAt: new Date().toISOString() },
    };
    return invoke<string>("export_config", { settings });
  }

  async function importConfig(json: string): Promise<void> {
    const settings = await invoke<AppSettings>("import_config", { json });
    appearance.value = settings.appearance;
    agents.value = settings.agents;
    repos.value = settings.repos ?? [];
    plugins.value = settings.plugins;
  }

  return {
    loaded,
    appearance,
    agents,
    repos,
    plugins,
    load,
    save,
    updateTheme,
    updateLanguage,
    updateFontSize,
    addAgent,
    removeAgent,
    updateAgent,
    addRepo,
    removeRepo,
    updateRepo,
    togglePlugin,
    updatePluginOrder,
    // Git Sync
    syncResults,
    syncing,
    syncAllRepos,
    getRemoteSkills,
    // Config Portability
    exportConfig,
    importConfig,
  };
});
