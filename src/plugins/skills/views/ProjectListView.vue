<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { NCard, NSpace, NText, NButton, NEmpty, NPopconfirm, NTag, NInput, NSpin, useMessage } from "naive-ui";
import { AddOutline, SyncOutline } from "@vicons/ionicons5";
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
  readme: string;
  loading: boolean;
}

const summaries = ref<ProjectSummary[]>([]);
const addingPath = ref("");
const showAddForm = ref(false);

const README_NAMES = ["README.md", "readme.md", "Readme.md", "README", "readme"];

async function tryReadReadme(projectPath: string): Promise<string> {
  for (const name of README_NAMES) {
    try {
      const content = await invoke<string>("read_skill_file", {
        filePath: `${projectPath}/${name}`,
      });
      if (content) return content.trim().split("\n").slice(0, 5).join("\n");
    } catch {
      // Not found, try next
    }
  }
  return "";
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) => setTimeout(() => reject(new Error("timeout")), ms)),
  ]);
}

async function loadProjectSummary(s: ProjectSummary) {
  // Load comparisons with timeout
  try {
    const localDir = store.resolveLocalDir("project", s.path);
    if (localDir) {
      s.comparisons = await withTimeout(
        invoke<SkillComparison[]>("compare_skills", {
          localDir,
          repos: settingsStore.repos,
        }),
        10000,
      );
    }
  } catch (e) {
    console.warn("[ProjectListView] compare_skills failed for", s.path, e);
  }
  // Load README with timeout
  try {
    s.readme = await withTimeout(tryReadReadme(s.path), 3000);
  } catch {
    // No readme
  }
  s.loading = false;
}

async function loadSummaries() {
  const results: ProjectSummary[] = [];
  for (const p of store.projectPaths) {
    const name = p.split(/[/\\]/).pop() ?? p;
    results.push({ path: p, name, comparisons: [], readme: "", loading: true });
  }
  summaries.value = results;

  // Load all projects concurrently
  await Promise.all(results.map(loadProjectSummary));
}

function handleSync() {
  settingsStore.syncAllRepos().then(() => {
    loadSummaries();
  });
}

onMounted(loadSummaries);

function getCounts(comparisons: SkillComparison[]) {
  return {
    outdated: comparisons.filter(c => c.status === "outdated").length,
    remoteOnly: comparisons.filter(c => c.status === "remoteOnly").length,
    localOnly: comparisons.filter(c => c.status === "localOnly").length,
    same: comparisons.filter(c => c.status === "same").length,
    total: comparisons.length,
  };
}

function goToDetail(path: string) {
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
          <NButton size="small" :loading="settingsStore.syncing" @click="handleSync">
            <template #icon><SyncOutline /></template>
            {{ settingsStore.syncing ? t("skills.sync.syncing") : t("skills.sync.syncAll") }}
          </NButton>
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

      <div v-if="summaries.length === 0 && !summaries.some(s => s.loading)">
        <NEmpty :description="t('skills.project.noProjects')" />
      </div>

      <!-- Project cards -->
      <NCard
        v-for="proj in summaries"
        :key="proj.path"
        hoverable
        style="cursor: pointer; margin-bottom: 12px"
        @click="goToDetail(proj.path)"
      >
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px">
          <NText strong style="font-size: 16px">{{ proj.name }}</NText>
          <NSpace v-if="!proj.loading" size="small" @click.stop>
            <NTag size="small" round>{{ getCounts(proj.comparisons).total }} 本地</NTag>
            <NTag v-if="getCounts(proj.comparisons).same > 0" size="small" type="success" round>
              {{ getCounts(proj.comparisons).same }} {{ t("skills.project.upToDate") }}
            </NTag>
            <NTag v-if="getCounts(proj.comparisons).outdated > 0" size="small" type="warning" round>
              {{ getCounts(proj.comparisons).outdated }} {{ t("skills.project.outdated") }}
            </NTag>
            <NTag v-if="getCounts(proj.comparisons).remoteOnly > 0" size="small" round>
              {{ getCounts(proj.comparisons).remoteOnly }} {{ t("skills.project.available") }}
            </NTag>
          </NSpace>
          <NSpin v-else size="small" />
        </div>
        <div style="margin-top: 6px">
          <NText depth="3" style="font-size: 12px; font-family: monospace; word-break: break-all">
            {{ proj.path }}
          </NText>
        </div>
        <div v-if="proj.readme" style="margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--n-border-color)">
          <NText depth="2" style="font-size: 13px; white-space: pre-line; max-height: 80px; overflow: hidden; display: block">
            {{ proj.readme }}
          </NText>
        </div>
        <template #action>
          <NSpace justify="end">
            <NPopconfirm @positive-click.stop="handleRemove(proj.path)">
              <template #trigger>
                <NButton size="tiny" type="error" ghost @click.stop>
                  {{ t("skills.project.remove") }}
                </NButton>
              </template>
              {{ t("skills.project.removeConfirm") }}
            </NPopconfirm>
            <NButton size="tiny" type="primary" @click.stop="goToDetail(proj.path)">
              {{ t("skills.project.detail") }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </NSpace>
  </div>
</template>
