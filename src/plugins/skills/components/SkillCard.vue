<script setup lang="ts">
import {
  NCard,
  NCheckbox,
  NSpace,
  NText,
  NTag,
  NButton,
  NTooltip,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import type { SkillInfo } from "../types";

defineProps<{ skill: SkillInfo }>();
defineEmits<{
  uninstall: [skill: SkillInfo];
}>();
const { t } = useI18n();
const store = useSkillsStore();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
</script>

<template>
  <NCard size="small" hoverable style="cursor: pointer">
    <template #header>
      <NSpace align="center" :wrap="false">
        <NCheckbox
          :checked="store.selectedSkills.has(skill.name)"
          @update:checked="store.toggleSelect(skill.name)"
          @click.stop
        />
        <NText strong>{{ skill.meta?.name ?? skill.name }}</NText>
        <NTag v-if="skill.meta?.version" size="small" type="info">
          v{{ skill.meta.version }}
        </NTag>
      </NSpace>
    </template>
    <NText depth="3" style="font-size: 13px">
      {{ skill.meta?.description ?? t("skills.card.noMeta") }}
    </NText>
    <template #action>
      <NSpace justify="space-between" align="center">
        <NSpace :size="12">
          <NText depth="3" style="font-size: 12px">
            {{ t("skills.card.files", { count: skill.fileCount }) }}
            · {{ formatSize(skill.sizeBytes) }}
          </NText>
        </NSpace>
        <NTooltip>
          <template #trigger>
            <NButton
              size="tiny"
              type="error"
              quaternary
              @click.stop="$emit('uninstall', skill)"
            >
              {{ t("skills.actions.uninstall") }}
            </NButton>
          </template>
          {{ skill.path }}
        </NTooltip>
      </NSpace>
    </template>
  </NCard>
</template>
