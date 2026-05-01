<script setup lang="ts">
import { computed } from "vue";
import { NSelect, NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";

const { t } = useI18n();
const store = useSkillsStore();

const agentOptions = computed(() =>
  store.availableAgents.map((agent) => ({
    label: agent.name,
    value: agent.id,
  }))
);

function handleChange(agentId: string) {
  store.switchAgent(agentId);
}
</script>

<template>
  <NSpace align="center" :wrap="false">
    <NText depth="3" style="font-size: 13px; white-space: nowrap">
      {{ t("skills.agent.current") }}
    </NText>
    <NSelect
      :value="store.currentAgentId"
      :options="agentOptions"
      size="small"
      :placeholder="t('skills.agent.select')"
      style="min-width: 180px"
      @update:value="handleChange"
    />
  </NSpace>
</template>
