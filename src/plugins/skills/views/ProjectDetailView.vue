<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NSpace, NText, NButton, NIcon } from "naive-ui";
import { ArrowBackOutline } from "@vicons/ionicons5";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import SkillCompareTable from "../components/SkillCompareTable.vue";
import SkillDiffViewer from "../components/SkillDiffViewer.vue";
import SkillPreviewModal from "../components/SkillPreviewModal.vue";

const route = useRoute();
const router = useRouter();
const store = useSkillsStore();

// Decode project path from base64 route param
const projectPath = computed(() => {
  try {
    return atob(route.params.encodedPath as string);
  } catch {
    return "";
  }
});

const projectName = computed(() => {
  return projectPath.value.split(/[/\\]/).pop() ?? projectPath.value;
});

const showDiff = ref(false);
const diffLocalPath = ref("");
const diffRemotePath = ref("");
const showPreview = ref(false);
const previewSkillPath = ref("");
const previewSkillName = ref("");

onMounted(() => {
  if (projectPath.value) {
    store.loadComparisons("project", projectPath.value);
  }
});

watch(() => store.currentAgentId, () => {
  if (projectPath.value) {
    store.loadComparisons("project", projectPath.value);
  }
});

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

function goBack() {
  router.push({ name: "skills-projects" });
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column">
    <NSpace vertical :size="16" style="flex: 1; overflow: auto">
      <NSpace align="center" :size="12">
        <NButton quaternary size="small" @click="goBack">
          <template #icon><NIcon :component="ArrowBackOutline" /></template>
        </NButton>
        <NText strong style="font-size: 18px">{{ projectName }}</NText>
      </NSpace>

      <NText depth="3" style="font-size: 12px; font-family: monospace">
        {{ projectPath }}
      </NText>

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
</template>
