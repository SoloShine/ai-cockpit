<script setup lang="ts">
import { ref } from "vue";
import { NCard, NSwitch, NSpace, NText, NIcon, NCollapse, NCollapseItem, NButton } from "naive-ui";
import { ChevronUpOutline, ChevronDownOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";
import { usePluginStore } from "@/stores/plugin";
import PluginDetail from "../components/PluginDetail.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const pluginStore = usePluginStore();
const expandedId = ref<string | null>(null);

function isEnabled(pluginId: string): boolean {
  return !settingsStore.plugins.disabledIds.includes(pluginId);
}

function toggleEnabled(pluginId: string, enabled: boolean) {
  if (pluginId === "settings") return;
  settingsStore.togglePlugin(pluginId, enabled);
}

function moveUp(index: number) {
  const plugins = [...pluginStore.plugins];
  const order = plugins.map((p) => p.id);
  if (index > 0) {
    [order[index], order[index - 1]] = [order[index - 1], order[index]];
    settingsStore.updatePluginOrder(order);
    pluginStore.refresh();
  }
}

function moveDown(index: number) {
  const plugins = [...pluginStore.plugins];
  const order = plugins.map((p) => p.id);
  if (index < plugins.length - 1) {
    [order[index], order[index + 1]] = [order[index + 1], order[index]];
    settingsStore.updatePluginOrder(order);
    pluginStore.refresh();
  }
}
</script>

<template>
  <div>
    <NText strong style="font-size: 16px; display: block; margin-bottom: 16px">
      {{ t("settings.plugins.title") }}
    </NText>

    <NCard
      v-for="(plugin, index) in pluginStore.plugins"
      :key="plugin.id"
      size="small"
      style="margin-bottom: 8px"
    >
      <NSpace align="center" justify="space-between">
        <NSpace align="center">
          <NIcon size="20"><component :is="plugin.icon" /></NIcon>
          <NText strong>{{ plugin.name }}</NText>
          <NText depth="3" v-if="plugin.description">{{ plugin.description }}</NText>
        </NSpace>
        <NSpace align="center">
          <NButton
            quaternary size="tiny"
            :disabled="index === 0"
            @click="moveUp(index)"
          >
            <template #icon><NIcon><ChevronUpOutline /></NIcon></template>
          </NButton>
          <NButton
            quaternary size="tiny"
            :disabled="index === pluginStore.plugins.length - 1"
            @click="moveDown(index)"
          >
            <template #icon><NIcon><ChevronDownOutline /></NIcon></template>
          </NButton>
          <NSwitch
            :value="isEnabled(plugin.id)"
            :disabled="plugin.id === 'settings'"
            @update:value="toggleEnabled(plugin.id, $event)"
          />
        </NSpace>
      </NSpace>

      <NCollapse v-model:expanded-names="expandedId">
        <NCollapseItem :name="plugin.id" title="">
          <PluginDetail :plugin="plugin" />
        </NCollapseItem>
      </NCollapse>
    </NCard>
  </div>
</template>
