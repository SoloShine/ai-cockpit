<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NDescriptions, NDescriptionsItem, NButton, NSpace, NText, NTag, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const message = useMessage();

const version = ref("");
const dataDir = ref("");

onMounted(async () => {
  version.value = await invoke<string>("get_app_version");
  dataDir.value = await invoke<string>("get_data_dir");
});

async function openDataDir() {
  try {
    await invoke("open_in_explorer", { path: dataDir.value });
  } catch (e) {
    message.error("无法打开目录: " + e);
  }
}

async function openLogs() {
  await openDataDir();
}

function checkUpdates() {
  message.info("检查更新功能将在后续版本中实现");
}
</script>

<template>
  <div>
    <NDescriptions label-placement="left" bordered :column="1" title="AI Cockpit">
      <NDescriptionsItem :label="t('settings.about.version')">
        <NTag type="success">v{{ version }}</NTag>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.dataDir')">
        <NSpace align="center">
          <NText code>{{ dataDir }}</NText>
          <NButton size="tiny" @click="openDataDir">
            {{ t("settings.about.openInExplorer") }}
          </NButton>
        </NSpace>
      </NDescriptionsItem>
      <NDescriptionsItem :label="t('settings.about.techStack')">
        <NSpace>
          <NTag>Tauri 2</NTag>
          <NTag>Vue 3</NTag>
          <NTag>TypeScript</NTag>
          <NTag>Naive UI</NTag>
          <NTag>Rust</NTag>
        </NSpace>
      </NDescriptionsItem>
    </NDescriptions>

    <NSpace style="margin-top: 16px">
      <NButton @click="checkUpdates">{{ t("settings.about.checkUpdates") }}</NButton>
      <NButton @click="openLogs">{{ t("settings.about.openLogs") }}</NButton>
    </NSpace>
  </div>
</template>
