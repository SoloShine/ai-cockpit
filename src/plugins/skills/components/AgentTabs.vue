<script setup lang="ts">
import { NScrollbar, NTab, NTabs } from "naive-ui";
import { useSkillsStore } from "../store";

const store = useSkillsStore();

function handleChange(agentId: string) {
  store.switchAgent(agentId);
}
</script>

<template>
  <NScrollbar x-scrollable>
    <NTabs
      :value="store.currentAgentId"
      type="line"
      size="small"
      @update:value="handleChange"
    >
      <NTab
        v-for="agent in store.availableAgents"
        :key="agent.id"
        :name="agent.id"
      >
        {{ agent.name }}
      </NTab>
    </NTabs>
  </NScrollbar>
</template>
