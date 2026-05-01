<script setup lang="ts">
import { NTabs, NTabPane } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";

const { t } = useI18n();
const store = useSkillsStore();

function handleChange(value: string) {
  store.switchScope(value as "global" | "project");
}
</script>

<template>
  <NTabs
    :value="store.currentScope"
    type="segment"
    size="small"
    @update:value="handleChange"
  >
    <NTabPane name="global" :tab="t('skills.scope.global')" />
    <NTabPane name="project" :tab="t('skills.scope.project')" />
  </NTabs>
</template>
