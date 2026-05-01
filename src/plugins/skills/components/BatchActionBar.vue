<script setup lang="ts">
import { NSpace, NText, NButton } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";

const { t } = useI18n();
const store = useSkillsStore();
</script>

<template>
  <NSpace
    v-if="store.selectedSkills.size > 0"
    align="center"
    justify="space-between"
    style="padding: 12px 0; border-top: 1px solid var(--n-border-color); margin-top: 12px"
  >
    <NText>
      {{ t("skills.status.selected", { count: store.selectedSkills.size }) }}
    </NText>
    <NSpace>
      <NButton size="small" @click="store.clearSelection()">
        {{ t("skills.actions.batch") }}
      </NButton>
      <NButton size="small" type="error" @click="store.clearSelection()">
        {{ t("skills.actions.uninstall") }}
      </NButton>
    </NSpace>
  </NSpace>
</template>
