<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NDescriptions, NDescriptionsItem, NButton, NSpace, NText, NTag, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../store";

const { t } = useI18n();
const message = useMessage();
const store = useSettingsStore();

const version = ref("loading...");
const dataDir = ref("loading...");

onMounted(async () => {
  try {
    version.value = await invoke<string>("get_app_version");
    dataDir.value = await invoke<string>("get_data_dir");
  } catch (e) {
    version.value = "error: " + String(e);
    dataDir.value = "error";
  }
});

async function openDataDir() {
  try {
    await invoke("open_in_explorer", { path: dataDir.value });
  } catch (e) {
    message.error("Cannot open directory: " + String(e));
  }
}

async function openLogs() {
  await openDataDir();
}

function checkUpdates() {
  message.info("Check for updates will be available in a future version");
}

async function handleExport() {
  try {
    const json = await store.exportConfig();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "ai-cockpit-settings.json";
    a.click();
    URL.revokeObjectURL(url);
    message.success("Config exported");
  } catch (e) {
    message.error("Export failed: " + String(e));
  }
}

async function handleImport() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".json";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const json = await file.text();
      await store.importConfig(json);
      message.success("Config imported");
    } catch (e) {
      message.error("Import failed: " + String(e));
    }
  };
  input.click();
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
      <NButton type="primary" ghost @click="handleExport">Export Config</NButton>
      <NButton ghost @click="handleImport">Import Config</NButton>
    </NSpace>
  </div>
</template>
