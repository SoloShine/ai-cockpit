<script setup lang="ts">
import { onMounted } from "vue";
import { NSpace, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import ScopeTabs from "../components/ScopeTabs.vue";
import ProjectSelector from "../components/ProjectSelector.vue";
import SkillList from "../components/SkillList.vue";
import BatchActionBar from "../components/BatchActionBar.vue";

const { t } = useI18n();
const store = useSkillsStore();

onMounted(() => {
  store.scanSkills(store.currentAgentId, store.currentScope);
});
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NText strong style="font-size: 18px">
        {{ t("skills.title") }}
      </NText>

      <AgentSelect />

      <ScopeTabs />

      <ProjectSelector v-if="store.currentScope === 'project'" />

      <SkillList />
    </NSpace>

    <BatchActionBar />
  </div>
</template>
