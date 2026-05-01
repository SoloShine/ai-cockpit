<script setup lang="ts">
import { onMounted } from "vue";
import { NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import ScopeTabs from "../components/ScopeTabs.vue";
import AgentTabs from "../components/AgentTabs.vue";
import SkillList from "../components/SkillList.vue";
import BatchActionBar from "../components/BatchActionBar.vue";

const { t } = useI18n();
const store = useSkillsStore();

onMounted(() => {
  const agentConfig = store.getCurrentAgentConfig();
  if (!agentConfig || (!agentConfig.globalPath && !agentConfig.projectPath)) {
    return;
  }
  store.scanAllAgents(store.currentScope);
});
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace align="center" justify="space-between">
        <NText strong style="font-size: 18px">
          {{ t("skills.title") }}
        </NText>
      </NSpace>

      <ScopeTabs />

      <AgentTabs />

      <SkillList />
    </NSpace>

    <BatchActionBar />
  </div>
</template>
