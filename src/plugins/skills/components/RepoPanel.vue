<script setup lang="ts">
import { ref, computed } from "vue";
import {
  NButton,
  NCard,
  NSpace,
  NText,
  NInput,
  NSwitch,
  NTag,
  NEmpty,
  useMessage,
  NPopconfirm,
} from "naive-ui";
import { AddOutline, SyncOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/plugins/settings/store";
import type { SyncResult } from "@/plugins/skills/types";

const { t } = useI18n();
const store = useSettingsStore();
const message = useMessage();

const showAddForm = ref(false);
const newName = ref("");
const newUrl = ref("");

const syncing = computed(() => store.syncing);
const syncResults = computed(() => store.syncResults);

function getSyncResult(repoId: string): SyncResult | undefined {
  return syncResults.value.find((r) => r.repoId === repoId);
}

async function handleSyncAll() {
  if (store.repos.filter((r) => r.enabled).length === 0) {
    message.info(t("skills.sync.noReposToSync"));
    return;
  }
  try {
    const results = await store.syncAllRepos();
    const success = results.filter((r) => r.success).length;
    const fail = results.filter((r) => !r.success).length;
    if (fail > 0) {
      message.warning(t("skills.sync.syncSuccess", { success, fail }));
    } else {
      message.success(t("skills.sync.syncSuccess", { success, fail: 0 }));
    }
  } catch (e) {
    message.error(t("skills.sync.syncFail") + ": " + String(e));
  }
}

function handleAdd() {
  if (!newName.value.trim()) {
    message.warning(t("skills.repos.nameRequired"));
    return;
  }
  if (!newUrl.value.trim()) {
    message.warning(t("skills.repos.urlRequired"));
    return;
  }
  const id = `repo_${Date.now()}`;
  store.addRepo({
    id,
    name: newName.value.trim(),
    url: newUrl.value.trim(),
    cachePath: `repos/${id}`,
    enabled: true,
  });
  newName.value = "";
  newUrl.value = "";
  showAddForm.value = false;
  message.success(t("skills.repos.addSuccess"));
}

function handleDelete(id: string) {
  store.removeRepo(id);
  message.success(t("skills.repos.deleteSuccess"));
}
</script>

<template>
  <div>
    <NSpace justify="space-between" align="center" style="margin-bottom: 16px">
      <NText strong style="font-size: 16px">{{ t("skills.repos.title") }}</NText>
      <NSpace>
        <NButton
          type="primary"
          ghost
          :loading="syncing"
          @click="handleSyncAll"
        >
          <template #icon><SyncOutline /></template>
          {{ syncing ? t("skills.sync.syncing") : t("skills.sync.syncAll") }}
        </NButton>
        <NButton @click="showAddForm = !showAddForm">
          <template #icon><AddOutline /></template>
          {{ t("skills.repos.addRepo") }}
        </NButton>
      </NSpace>
    </NSpace>

    <NCard v-if="showAddForm" size="small" style="margin-bottom: 16px">
      <NSpace vertical :size="12">
        <NInput
          v-model:value="newName"
          :placeholder="t('skills.repos.name')"
          size="small"
        />
        <NInput
          v-model:value="newUrl"
          :placeholder="t('skills.repos.url')"
          size="small"
        />
        <NSpace>
          <NButton size="small" type="primary" @click="handleAdd">
            {{ t("skills.repos.addRepo") }}
          </NButton>
          <NButton size="small" @click="showAddForm = false">
            {{ t("skills.repos.cancel") }}
          </NButton>
        </NSpace>
      </NSpace>
    </NCard>

    <NEmpty v-if="store.repos.length === 0" :description="t('skills.repos.noRepos')" />

    <NCard
      v-for="repo in store.repos"
      :key="repo.id"
      size="small"
      style="margin-bottom: 12px"
    >
      <template #header>
        <NSpace align="center">
          <span>{{ repo.name }}</span>
          <NTag :type="repo.enabled ? 'success' : 'default'" size="small">
            {{ repo.enabled ? t("skills.repos.enabled") : t("skills.repos.disabled") }}
          </NTag>
          <NTag
            v-if="getSyncResult(repo.id)"
            :type="getSyncResult(repo.id)!.success ? 'success' : 'error'"
            size="small"
          >
            {{ getSyncResult(repo.id)!.success
              ? t("skills.sync.skillCount", { count: getSyncResult(repo.id)!.skillCount })
              : t("skills.sync.syncFail")
            }}
          </NTag>
        </NSpace>
      </template>
      <NSpace vertical :size="8">
        <NText depth="3" style="font-size: 13px; word-break: break-all">
          {{ repo.url }}
        </NText>
        <NSpace align="center" justify="space-between">
          <NSwitch
            :value="repo.enabled"
            @update:value="store.updateRepo(repo.id, { enabled: $event })"
          >
            <template #checked>{{ t("skills.repos.enabled") }}</template>
          </NSwitch>
          <NPopconfirm @positive-click="handleDelete(repo.id)">
            <template #trigger>
              <NButton size="tiny" type="error" quaternary>
                {{ t("skills.repos.delete") }}
              </NButton>
            </template>
            {{ t("skills.repos.deleteConfirm") }}
          </NPopconfirm>
        </NSpace>
      </NSpace>
    </NCard>
  </div>
</template>
