<script setup lang="ts">
import { ref, computed } from "vue";
import { NTabs, NTabPane } from "naive-ui";
import { useI18n } from "vue-i18n";
import { pluginRegistry } from "@/core/plugin";
import AppearancePanel from "../panels/AppearancePanel.vue";
import AgentPanel from "../panels/AgentPanel.vue";
import PluginPanel from "../panels/PluginPanel.vue";
import AboutPanel from "../panels/AboutPanel.vue";

const { t } = useI18n();
const activeTab = ref("appearance");

const pluginSettingsTabs = computed(() => {
  return pluginRegistry.getAll()
    .filter((p) => {
      if (p.id === "settings") return false;
      const hooks = pluginRegistry.getHooks(p.id);
      return !!hooks?.SettingsPanel;
    })
    .map((p) => ({
      id: p.id,
      name: p.name,
      component: pluginRegistry.getHooks(p.id)!.SettingsPanel!,
    }));
});
</script>

<template>
  <div style="height: 100%">
    <NTabs
      v-model:value="activeTab"
      type="line"
      placement="left"
      style="height: 100%"
      :tabs-padding="24"
    >
      <NTabPane name="appearance" :tab="t('settings.tabs.appearance')">
        <AppearancePanel />
      </NTabPane>
      <NTabPane name="agents" :tab="t('settings.tabs.agents')">
        <AgentPanel />
      </NTabPane>
      <NTabPane
        v-for="tab in pluginSettingsTabs"
        :key="tab.id"
        :name="`plugin-${tab.id}`"
        :tab="tab.name"
      >
        <component :is="tab.component" />
      </NTabPane>
      <NTabPane name="plugins" :tab="t('settings.tabs.plugins')">
        <PluginPanel />
      </NTabPane>
      <NTabPane name="about" :tab="t('settings.tabs.about')">
        <AboutPanel />
      </NTabPane>
    </NTabs>
  </div>
</template>
