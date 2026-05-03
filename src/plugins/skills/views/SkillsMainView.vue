<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { NSpace, NText, NButton, NIcon } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import SkillCompareTable from "../components/SkillCompareTable.vue";
import SkillDiffViewer from "../components/SkillDiffViewer.vue";
import SkillPreviewModal from "../components/SkillPreviewModal.vue";
import OperationHistoryPanel from "../components/OperationHistoryPanel.vue";
import MigrateDialog from "../components/MigrateDialog.vue";
import { SyncOutline, TimeOutline, SwapHorizontalOutline } from "@vicons/ionicons5";
import { useSettingsStore } from "@/plugins/settings/store";

const { t } = useI18n();
const store = useSkillsStore();
const settingsStore = useSettingsStore();

const showDiff = ref(false);
const diffLocalPath = ref("");
const diffRemotePath = ref("");
const showPreview = ref(false);
const previewSkillPath = ref("");
const previewSkillName = ref("");

onMounted(() => {
  store.currentScope = "global";
  store.loadComparisons();
});

watch(() => store.currentAgentId, () => {
  store.loadComparisons();
});

function handleSync() {
  settingsStore.syncAllRepos().then(() => {
    store.loadComparisons();
  });
}

function handleDiff(localPath: string, remotePath: string) {
  diffLocalPath.value = localPath;
  diffRemotePath.value = remotePath;
  showDiff.value = true;
}

function handlePreview(skillPath: string, skillName: string) {
  previewSkillPath.value = skillPath;
  previewSkillName.value = skillName;
  showPreview.value = true;
}
</script>

<template>
  <div data-testid="skills-main-page" style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace justify="space-between" align="center">
        <NText strong style="font-size: 18px">{{ t("skills.title") }}</NText>
        <NSpace size="small">
          <NButton size="small" :loading="settingsStore.syncing" @click="handleSync">
            <template #icon><NIcon :component="SyncOutline" /></template>
            {{ settingsStore.syncing ? t("skills.sync.syncing") : t("skills.sync.syncAll") }}
          </NButton>
          <NButton size="small" data-testid="btn-history" @click="store.showHistoryPanel = true">
            <template #icon><NIcon :component="TimeOutline" /></template>
            {{ t("skills.history.title") }}
          </NButton>
          <NButton size="small" data-testid="btn-migrate" @click="store.showMigrateDialog = true">
            <template #icon><NIcon :component="SwapHorizontalOutline" /></template>
            {{ t("skills.migrate.title") }}
          </NButton>
        </NSpace>
      </NSpace>

      <AgentSelect />

      <SkillCompareTable @diff="handleDiff" @preview="handlePreview" />
    </NSpace>
  </div>

  <SkillDiffViewer
    :show="showDiff"
    :local-path="diffLocalPath"
    :remote-path="diffRemotePath"
    @close="showDiff = false"
  />

  <SkillPreviewModal
    :show="showPreview"
    :skill-path="previewSkillPath"
    :skill-name="previewSkillName"
    @close="showPreview = false"
  />

  <OperationHistoryPanel
    :show="store.showHistoryPanel"
    @close="store.showHistoryPanel = false"
  />

  <MigrateDialog />
</template>
