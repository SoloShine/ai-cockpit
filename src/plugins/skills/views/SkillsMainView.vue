<script setup lang="ts">
import { onMounted, ref, h } from "vue";
import { NSpace, NText, NButton, NIcon } from "naive-ui";
import { useI18n } from "vue-i18n";
import { GitCompareOutline } from "@vicons/ionicons5";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import ScopeTabs from "../components/ScopeTabs.vue";
import ProjectSelector from "../components/ProjectSelector.vue";
import SkillList from "../components/SkillList.vue";
import BatchActionBar from "../components/BatchActionBar.vue";
import SkillCompareTable from "../components/SkillCompareTable.vue";
import SkillDiffViewer from "../components/SkillDiffViewer.vue";
import SkillPreviewModal from "../components/SkillPreviewModal.vue";

const { t } = useI18n();
const store = useSkillsStore();

// Modal state
const showDiff = ref(false);
const diffLocalPath = ref("");
const diffRemotePath = ref("");
const showPreview = ref(false);
const previewSkillPath = ref("");
const previewSkillName = ref("");

// Event handlers
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

onMounted(() => {
  store.scanSkills(store.currentAgentId, store.currentScope);
});
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <div style="display: flex; justify-content: space-between; align-items: center">
        <NText strong style="font-size: 18px">
          {{ t("skills.title") }}
        </NText>
        <NButton
          secondary
          size="small"
          :icon="() => h(NIcon, null, { default: () => h(GitCompareOutline) })"
          @click="store.toggleComparisonMode()"
        >
          {{ store.comparisonMode ? t("skills.compare.modeOn") : t("skills.compare.modeOff") }}
        </NButton>
      </div>

      <AgentSelect />

      <ScopeTabs />

      <ProjectSelector v-if="store.currentScope === 'project'" />

      <SkillCompareTable
        v-if="store.comparisonMode"
        @diff="handleDiff"
        @preview="handlePreview"
      />
      <SkillList v-else />
    </NSpace>

    <BatchActionBar v-if="!store.comparisonMode" />
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
</template>
