<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { NSpace, NText, NButton, NIcon, NInput, useMessage } from "naive-ui";
import { ArrowBackOutline, SearchOutline } from "@vicons/ionicons5";
import { useSkillsStore } from "../store";
import AgentSelect from "../components/AgentSelect.vue";
import SkillCompareTable from "../components/SkillCompareTable.vue";
import SkillDiffViewer from "../components/SkillDiffViewer.vue";
import SkillPreviewModal from "../components/SkillPreviewModal.vue";
import SkillbasePanel from "../components/SkillbasePanel.vue";
import StatusFilterBar from "../components/StatusFilterBar.vue";

const route = useRoute();
const router = useRouter();
const store = useSkillsStore();
const { t } = useI18n();
const message = useMessage();

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

const searchInputRef = ref<InstanceType<typeof NInput> | null>(null);

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === "k") {
    e.preventDefault();
    const el = searchInputRef.value?.$el as HTMLElement | undefined;
    el?.querySelector("input")?.focus();
  }
}

const skillDir = computed(() => store.resolveLocalDir("project", projectPath.value));

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
  if (projectPath.value) {
    store.currentScope = "project";
    store.currentProjectPath = projectPath.value;
    store.loadComparisons();
  }
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});

watch(() => store.currentAgentId, () => {
  if (projectPath.value) {
    store.loadComparisons();
    if (store.showSkillbasePanel) {
      store.loadSkillbase(skillDir.value);
    }
  }
});

async function toggleSkillbase() {
  store.showSkillbasePanel = !store.showSkillbasePanel;
  if (store.showSkillbasePanel && !store.skillbase && skillDir.value) {
    await store.loadSkillbase(skillDir.value);
  }
}

async function handleSkillbaseSync() {
  if (!skillDir.value) return;
  const results = await store.syncSkillbaseDeps(skillDir.value);
  const failed = results.filter(r => !r.success);
  const succeeded = results.filter(r => r.success);
  if (failed.length === 0) {
    message.success(t('skills.skillbase.syncSuccess'));
  } else {
    message.warning(t('skills.skillbase.syncPartial', { success: succeeded.length, failed: failed.length }));
  }
}

async function handleSkillbaseRegenerate() {
  if (!skillDir.value) return;
  try {
    const content = await store.generateSkillbaseJson(skillDir.value);
    await store.writeSkillbaseJson(projectPath.value, content);
    await store.loadSkillbase(skillDir.value);
    message.success(t('skills.skillbase.generateSuccess'));
  } catch (e) {
    message.error(String(e));
  }
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
        <NButton
          size="tiny"
          :type="store.showSkillbasePanel ? 'primary' : 'default'"
          quaternary
          @click="toggleSkillbase"
        >
          {{ store.showSkillbasePanel ? t('skills.skillbase.hideSkillbase') : t('skills.skillbase.showSkillbase') }}
        </NButton>
      </NSpace>

      <NText depth="3" style="font-size: 12px; font-family: monospace">
        {{ projectPath }}
      </NText>

      <SkillbasePanel
        v-if="store.showSkillbasePanel && store.skillbase"
        :resolution="store.skillbase"
        :syncing="store.skillbaseSyncing"
        @sync="handleSkillbaseSync"
        @regenerate="handleSkillbaseRegenerate"
      />

      <AgentSelect />

      <NSpace align="center" justify="space-between" style="width: 100%">
        <StatusFilterBar />
        <NInput
          ref="searchInputRef"
          v-model:value="store.searchText"
          :placeholder="t('skills.filter.search')"
          data-testid="project-search-input"
          clearable
          style="width: 220px"
        >
          <template #prefix>
            <NIcon :component="SearchOutline" />
          </template>
        </NInput>
      </NSpace>

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
