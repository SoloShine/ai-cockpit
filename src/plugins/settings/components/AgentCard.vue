<script setup lang="ts">
import { NCard, NSwitch, NButton, NInput, NTag, NSpace, NIcon, NPopconfirm } from "naive-ui";
import { FolderOpenOutline, TrashOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import type { AgentConfig } from "../types";

defineProps<{ agent: AgentConfig }>();
const emit = defineEmits<{
  "update:agent": [updates: Partial<AgentConfig>];
  delete: [];
}>();
const { t } = useI18n();

async function browsePath() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    emit("update:agent", { basePath: selected });
  }
}
</script>

<template>
  <NCard size="small" style="margin-bottom: 12px">
    <template #header>
      <NSpace align="center">
        <span>{{ agent.name }}</span>
        <NTag :type="agent.isCustom ? 'info' : 'default'" size="small">
          {{ agent.isCustom ? t("settings.agents.custom") : t("settings.agents.builtIn") }}
        </NTag>
      </NSpace>
    </template>
    <NSpace vertical>
      <NSpace align="center">
        <NInput
          :value="agent.basePath"
          :placeholder="t('settings.agents.basePath')"
          style="flex: 1"
          @update:value="emit('update:agent', { basePath: $event })"
        />
        <NButton @click="browsePath">
          <template #icon><NIcon><FolderOpenOutline /></NIcon></template>
          {{ t("settings.agents.browse") }}
        </NButton>
      </NSpace>
      <NSpace justify="space-between" align="center">
        <NSwitch
          :value="agent.enabled"
          @update:value="emit('update:agent', { enabled: $event })"
        >
          <template #checked>{{ t("settings.agents.enabled") }}</template>
        </NSwitch>
        <NPopconfirm v-if="agent.isCustom" @positive-click="emit('delete')">
          <template #trigger>
            <NButton type="error" size="small" quaternary>
              <template #icon><NIcon><TrashOutline /></NIcon></template>
              {{ t("settings.agents.delete") }}
            </NButton>
          </template>
          {{ t("settings.agents.deleteConfirm") }}
        </NPopconfirm>
      </NSpace>
    </NSpace>
  </NCard>
</template>
