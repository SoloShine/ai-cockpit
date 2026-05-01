<script setup lang="ts">
import { NGrid, NGridItem, NSpin, NText } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import SkillCard from "./SkillCard.vue";
import EmptyState from "./EmptyState.vue";
import type { SkillInfo } from "../types";

const { t } = useI18n();
const store = useSkillsStore();

function handleUninstall(skill: SkillInfo) {
  store.uninstallSkill(skill.path);
}
</script>

<template>
  <NSpin :show="store.loading">
    <template v-if="!store.loading && store.currentSkills.length === 0">
      <EmptyState
        :type="store.error ? 'error' : 'noSkills'"
      />
    </template>
    <template v-else>
      <NText depth="3" style="margin-bottom: 12px; display: block; font-size: 13px">
        {{ t("skills.status.total", { count: store.currentSkills.length }) }}
      </NText>
      <NGrid :cols="2" :x-gap="12" :y-gap="12" responsive="screen">
        <NGridItem
          v-for="skill in store.currentSkills"
          :key="skill.name"
        >
          <SkillCard :skill="skill" @uninstall="handleUninstall" />
        </NGridItem>
      </NGrid>
    </template>
  </NSpin>
</template>
