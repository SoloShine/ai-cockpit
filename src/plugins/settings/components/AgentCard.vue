<script setup lang="ts">
import { NCard, NSwitch, NInput, NTag, NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import type { AgentConfig } from "../types";

defineProps<{ agent: AgentConfig }>();
const emit = defineEmits<{
  "update:agent": [updates: Partial<AgentConfig>];
  delete: [];
}>();
const { t } = useI18n();
</script>

<template>
  <NCard size="small">
    <template #header>
      <NSpace align="center">
        <span>{{ agent.name }}</span>
        <NTag :type="agent.isCustom ? 'info' : 'default'" size="small">
          {{ agent.isCustom ? t("settings.agents.custom") : t("settings.agents.builtIn") }}
        </NTag>
      </NSpace>
    </template>
    <NSpace vertical :size="12">
      <NSpace align="center" :wrap="false">
        <NText style="width: 70px; flex-shrink: 0">{{ t("settings.agents.globalPath") }}</NText>
        <NInput
          :value="agent.globalPath"
          :placeholder="t('settings.agents.globalPathPlaceholder')"
          size="small"
          @update:value="emit('update:agent', { globalPath: $event })"
        />
      </NSpace>
      <NSpace align="center" :wrap="false">
        <NText style="width: 70px; flex-shrink: 0">{{ t("settings.agents.projectPath") }}</NText>
        <NInput
          :value="agent.projectPath"
          :placeholder="t('settings.agents.projectPathPlaceholder')"
          size="small"
          @update:value="emit('update:agent', { projectPath: $event })"
        />
      </NSpace>
      <NSwitch
        :value="agent.enabled"
        @update:value="emit('update:agent', { enabled: $event })"
      >
        <template #checked>{{ t("settings.agents.enabled") }}</template>
      </NSwitch>
    </NSpace>
  </NCard>
</template>
