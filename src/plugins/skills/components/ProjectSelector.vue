<script setup lang="ts">
import { computed } from "vue";
import {
  NSelect,
  NSpace,
  NButton,
  NTag,
  NText,
  NEmpty,
} from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import { open } from "@tauri-apps/plugin-dialog";

const { t } = useI18n();
const store = useSkillsStore();

const projectOptions = computed(() =>
  store.projectPaths.map((p) => {
    const name = p.split(/[\\/]/).filter(Boolean).pop() ?? p;
    return { label: `${name} (${p})`, value: p };
  })
);

async function handleAddProject() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      store.addProject(selected);
      await store.selectProject(selected);
    }
  } catch (e) {
    console.error("[ProjectSelector] Failed to open directory dialog:", e);
  }
}

function handleSelect(path: string) {
  store.selectProject(path);
}

function handleRemove(path: string) {
  store.removeProject(path);
}
</script>

<template>
  <div>
    <NSpace align="center" justify="space-between" style="margin-bottom: 12px">
      <NText depth="3" style="font-size: 13px">
        {{ t("skills.project.selectProject") }}
      </NText>
      <NButton size="small" @click="handleAddProject">
        <template #icon><AddOutline /></template>
        {{ t("skills.project.addProject") }}
      </NButton>
    </NSpace>

    <NEmpty
      v-if="store.projectPaths.length === 0"
      :description="t('skills.project.noProjects')"
      style="margin: 24px 0"
    />

    <template v-else>
      <NSelect
        :value="store.currentProjectPath"
        :options="projectOptions"
        size="small"
        :placeholder="t('skills.project.selectProject')"
        style="margin-bottom: 12px"
        @update:value="handleSelect"
      />

      <NSpace :size="8" :wrap="true">
        <NTag
          v-for="path in store.projectPaths"
          :key="path"
          size="small"
          closable
          @close="handleRemove(path)"
        >
          {{ path.split(/[\\/]/).filter(Boolean).pop() }}
        </NTag>
      </NSpace>
    </template>
  </div>
</template>
