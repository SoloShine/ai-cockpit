import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { CockpitPlugin } from "@/core/plugin";
import { pluginRegistry } from "@/core/plugin";

export const usePluginStore = defineStore("plugin", () => {
  const plugins = ref<CockpitPlugin[]>([]);

  /** Refresh plugin list from registry. */
  function refresh() {
    plugins.value = pluginRegistry.getAll();
  }

  const activePluginId = computed(() => pluginRegistry.activeId);

  const navGroups = computed(() => pluginRegistry.getNavItems());

  return {
    plugins,
    activePluginId,
    navGroups,
    refresh,
  };
});
