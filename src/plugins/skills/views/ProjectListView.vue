<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { NCard, NSpace, NText, NButton, NEmpty, NPopconfirm, NTag, NInput, NSkeleton, useMessage } from "naive-ui";
import { AddOutline, SyncOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import { useSettingsStore } from "@/plugins/settings/store";

const { t } = useI18n();
const router = useRouter();
const store = useSkillsStore();
const settingsStore = useSettingsStore();
const message = useMessage();

const addingPath = ref("");
const showAddForm = ref(false);
import { ref } from "vue";

/** Format ISO timestamp to relative time string */
function formatRelativeTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffSec < 60) return t("skills.project.justNow");
    if (diffMin < 60) return t("skills.project.minutesAgo", { count: diffMin });
    if (diffHour < 24) return t("skills.project.hoursAgo", { count: diffHour });
    if (diffDay < 30) return t("skills.project.daysAgo", { count: diffDay });
    return date.toLocaleDateString();
  } catch {
    return isoString;
  }
}

async function loadSummaries() {
  await store.loadProjectsOverview();
}

function handleSync() {
  settingsStore.syncAllRepos().then(() => {
    loadSummaries();
  });
}

onMounted(loadSummaries);

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

/** Total skills for the status bar calculation */
function getTotal(proj: { localCount: number; remoteOnlyCount: number }): number {
  return proj.localCount + proj.remoteOnlyCount;
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

      <!-- Loading skeleton -->
      <template v-if="store.loadingProjects">
        <NSkeleton text style="height: 120px" />
        <NSkeleton text style="height: 120px" />
      </template>

      <!-- Empty state -->
      <div v-else-if="store.projectsOverview.length === 0">
        <NEmpty :description="t('skills.project.noProjects')" />
      </div>

      <!-- Project cards -->
      <NCard
        v-for="proj in store.projectsOverview"
        :key="proj.projectPath"
        hoverable
        style="cursor: pointer; margin-bottom: 12px"
        @click="goToDetail(proj.projectPath)"
      >
        <!-- Header: name + last updated -->
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px">
          <NText strong style="font-size: 16px">{{ proj.projectName }}</NText>
          <NText v-if="proj.lastModified" depth="3" style="font-size: 12px">
            {{ t("skills.project.lastUpdated", { time: formatRelativeTime(proj.lastModified) }) }}
          </NText>
        </div>

        <!-- Path -->
        <div style="margin-top: 4px">
          <NText depth="3" style="font-size: 12px; font-family: monospace; word-break: break-all">
            {{ proj.projectPath }}
          </NText>
        </div>

        <!-- Status bar (colored proportion bar) -->
        <div
          v-if="getTotal(proj) > 0"
          style="margin-top: 10px; display: flex; gap: 2px; height: 6px; border-radius: 3px; overflow: hidden; background: var(--n-color-hover)"
        >
          <div
            v-if="proj.sameCount > 0"
            :style="{ flex: proj.sameCount, background: '#18a058', borderRadius: '3px' }"
          />
          <div
            v-if="proj.outdatedCount > 0"
            :style="{ flex: proj.outdatedCount, background: '#f0a020', borderRadius: '3px' }"
          />
          <div
            v-if="proj.remoteOnlyCount > 0"
            :style="{ flex: proj.remoteOnlyCount, background: '#909399', borderRadius: '3px' }"
          />
        </div>

        <!-- Count tags -->
        <NSpace size="small" style="margin-top: 8px" @click.stop>
          <NTag size="small" round>{{ proj.localCount }} {{ t("skills.project.localLabel") }}</NTag>
          <NTag v-if="proj.sameCount > 0" size="small" type="success" round>
            {{ proj.sameCount }} {{ t("skills.project.upToDate") }}
          </NTag>
          <NTag v-if="proj.outdatedCount > 0" size="small" type="warning" round>
            {{ proj.outdatedCount }} {{ t("skills.project.outdated") }}
          </NTag>
          <NTag v-if="proj.remoteOnlyCount > 0" size="small" round>
            {{ proj.remoteOnlyCount }} {{ t("skills.project.available") }}
          </NTag>
          <NText v-if="proj.localCount === 0 && proj.remoteOnlyCount === 0" depth="3" style="font-size: 12px">
            {{ t("skills.project.noSkills") }}
          </NText>
        </NSpace>

        <!-- README preview -->
        <div v-if="proj.readmePreview" style="margin-top: 10px; padding-top: 8px; border-top: 1px solid var(--n-border-color)">
          <NText depth="2" style="font-size: 13px; white-space: pre-line; max-height: 80px; overflow: hidden; display: block">
            {{ proj.readmePreview }}
          </NText>
        </div>

        <template #action>
          <NSpace justify="end">
            <NPopconfirm @positive-click.stop="handleRemove(proj.projectPath)">
              <template #trigger>
                <NButton size="tiny" type="error" ghost @click.stop>
                  {{ t("skills.project.remove") }}
                </NButton>
              </template>
              {{ t("skills.project.removeConfirm") }}
            </NPopconfirm>
            <NButton size="tiny" type="primary" @click.stop="goToDetail(proj.projectPath)">
              {{ t("skills.project.detail") }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </NSpace>
  </div>
</template>
