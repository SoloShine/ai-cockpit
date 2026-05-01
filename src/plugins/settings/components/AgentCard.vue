<script setup lang="ts">
import { NCard, NSwitch, NInput, NTag, NSpace } from "naive-ui";
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
      </NSpace>
      <NSpace justify="space-between" align="center">
        <NSwitch
          :value="agent.enabled"
          @update:value="emit('update:agent', { enabled: $event })"
        />
      </NSpace>
    </NSpace>
  </NCard>
</template>
