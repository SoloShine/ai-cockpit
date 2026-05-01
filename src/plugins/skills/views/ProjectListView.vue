<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { NCard, NSpace, NText, NButton, NEmpty, NPopconfirm, NTag, NInput, useMessage } from "naive-ui";
import { AddOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/plugins/settings/store";
import type { SkillComparison } from "../types";

const { t } = useI18n();
const router = useRouter();
const store = useSkillsStore();
const settingsStore = useSettingsStore();
const message = useMessage();

interface ProjectSummary {
  path: string;
  name: string;
  comparisons: SkillComparison[];
  loading: boolean;
}

const summaries = ref<ProjectSummary[]>([]);
const addingPath = ref("");
const showAddForm = ref(false);

async function loadSummaries() {
  const results: ProjectSummary[] = [];
  for (const p of store.projectPaths) {
    const name = p.split(/[/\\]/).pop() ?? p;
    const summary: ProjectSummary = { path: p, name, comparisons: [], loading: true };
    results.push(summary);
  }
  summaries.value = results;

  // Load comparisons for each project in parallel
  await Promise.all(results.map(async (s) => {
    try {
      const localDir = store.resolveLocalDir("project", s.path);
      if (!localDir) { s.loading = false; return; }
      s.comparisons = await invoke<SkillComparison[]>("compare_skills", {
        localDir,
        repos: settingsStore.repos,
      });
    } catch {
      // Project may not have skills dir, that's fine
    } finally {
      s.loading = false;
    }
  }));
}

onMounted(loadSummaries);

function getCounts(comparisons: SkillComparison[]) {
  return {
    outdated: comparisons.filter(c => c.status === "outdated").length,
    remoteOnly: comparisons.filter(c => c.status === "remoteOnly").length,
    localOnly: comparisons.filter(c => c.status === "localOnly").length,
    same: comparisons.filter(c => c.status === "same").length,
  };
}

function goToDetail(path: string) {
  // Encode path to be URL-safe (base64)
  const encoded = btoa(path);
  router.push({ name: "skills-project-detail", params: { encodedPath: encoded } });
}

function toggleAddForm() {
  showAddForm.value = !showAddForm.value;
  addingPath.value = "";
}

function handleAddProject() {
  if (!addingPath.value.trim()) return;
  store.addProject(addingPath.value.trim());
  addingPath.value = "";
  showAddForm.value = false;
  loadSummaries();
  message.success(t("skills.project.addSuccess"));
}

function handleRemove(path: string) {
  store.removeProject(path);
  loadSummaries();
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace justify="space-between" align="center">
        <NText strong style="font-size: 18px">{{ t("skills.project.title") }}</NText>
        <NSpace>
          <NButton size="small" @click="toggleAddForm">
            <template #icon><AddOutline /></template>
            {{ t("skills.project.addProject") }}
          </NButton>
        </NSpace>
      </NSpace>

      <!-- Add form -->
      <NCard v-if="showAddForm" size="small">
        <NSpace :size="12" vertical style="width: 100%">
          <NText>{{ t("skills.project.pathLabel") }}</NText>
          <NInput
            v-model:value="addingPath"
            :placeholder="t('skills.project.pathPlaceholder')"
            @keyup.enter="handleAddProject"
          />
          <NSpace>
            <NButton size="small" type="primary" @click="handleAddProject">{{ t("skills.project.addProject") }}</NButton>
            <NButton size="small" @click="toggleAddForm">{{ t("skills.repos.cancel") }}</NButton>
          </NSpace>
        </NSpace>
      </NCard>

      <NEmpty v-if="summaries.length === 0" :description="t('skills.project.noProjects')" />

      <!-- Project cards -->
      <NCard
        v-for="proj in summaries"
        :key="proj.path"
        hoverable
        style="cursor: pointer; margin-bottom: 12px"
        @click="goToDetail(proj.path)"
      >
        <template #header>
          <NSpace align="center" :wrap="false">
            <NText strong>{{ proj.name }}</NText>
            <NSpace v-if="!proj.loading" :size="6">
              <NTag v-if="getCounts(proj.comparisons).outdated > 0" type="warning" size="small">
                {{ getCounts(proj.comparisons).outdated }} {{ t("skills.project.outdated") }}
              </NTag>
              <NTag v-if="getCounts(proj.comparisons).remoteOnly > 0" size="small">
                {{ getCounts(proj.comparisons).remoteOnly }} {{ t("skills.project.available") }}
              </NTag>
              <NTag v-if="getCounts(proj.comparisons).same > 0" type="success" size="small">
                {{ getCounts(proj.comparisons).same }} {{ t("skills.project.upToDate") }}
              </NTag>
            </NSpace>
          </NSpace>
        </template>
        <NText depth="3" style="font-size: 12px; font-family: monospace; word-break: break-all">
          {{ proj.path }}
        </NText>
        <template #action>
          <NSpace justify="end">
            <NPopconfirm @positive-click.stop="handleRemove(proj.path)">
              <template #trigger>
                <NButton size="tiny" type="error" quaternary @click.stop>
                  {{ t("skills.project.remove") }}
                </NButton>
              </template>
              {{ t("skills.project.removeConfirm") }}
            </NPopconfirm>
          </NSpace>
        </template>
      </NCard>
    </NSpace>
  </div>
</template>
