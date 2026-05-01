<script setup lang="ts">
import { ref } from "vue";
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
import { AddOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/plugins/settings/store";

const { t } = useI18n();
const store = useSettingsStore();
const message = useMessage();

const showAddForm = ref(false);
const newName = ref("");
const newUrl = ref("");

function handleAdd() {
  if (!newName.value.trim()) {
    message.warning(t("skills.repos.nameRequired"));
    return;
  }
  if (!newUrl.value.trim()) {
    message.warning(t("skills.repos.urlRequired"));
    return;
  }
  store.addRepo({
    id: `repo_${Date.now()}`,
    name: newName.value.trim(),
    url: newUrl.value.trim(),
    cachePath: "",
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
      <NButton type="primary" @click="showAddForm = !showAddForm">
        <template #icon><AddOutline /></template>
        {{ t("skills.repos.addRepo") }}
      </NButton>
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
