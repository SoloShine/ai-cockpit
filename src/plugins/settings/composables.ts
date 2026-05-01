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
