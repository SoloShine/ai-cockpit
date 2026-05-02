// src/plugins/skills/store.ts
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/plugins/settings/store";
import type {
  SkillInfo,
  SkillScope,
  ScanResult,
  OperationResult,
  SkillOperation,
  SkillComparison,
  SkillDiffResult,
  DiffFileContent,
  OperationRecord,
} from "./types";

const PROJECT_PATHS_KEY = "skills_project_paths";

function loadProjectPaths(): string[] {
  try {
    const raw = localStorage.getItem(PROJECT_PATHS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveProjectPaths(paths: string[]) {
  localStorage.setItem(PROJECT_PATHS_KEY, JSON.stringify(paths));
}

export const useSkillsStore = defineStore("skills", () => {
  // State
  const currentAgentId = ref<string>("claude-code");
  const currentScope = ref<SkillScope>("global");
  const currentProjectPath = ref<string>("");
  const projectPaths = ref<string[]>(loadProjectPaths());
  const globalSkills = ref<Map<string, ScanResult>>(new Map());
  const projectSkills = ref<Map<string, ScanResult>>(new Map());
  const selectedSkills = ref<Set<string>>(new Set());
  const loading = ref(false);
  const error = ref<string | null>(null);
  const comparisons = ref<SkillComparison[]>([]);
  const comparisonMode = ref(false);
  const currentDiff = ref<SkillDiffResult | null>(null);
  const loadingDiff = ref(false);
  const operationHistory = ref<OperationRecord[]>([]);
  const showHistoryPanel = ref(false);

  // Computed
  const currentSkills = computed<SkillInfo[]>(() => {
    const skillsMap =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    const result = skillsMap.get(currentAgentId.value);
    return result?.skills ?? [];
  });

  const availableAgents = computed(() => {
    const store = useSettingsStore();
    return store.agents.filter((a) => a.enabled);
  });

  const comparisonCounts = computed(() => {
    const counts = { outdated: 0, remoteOnly: 0, localOnly: 0, same: 0 };
    for (const c of comparisons.value) {
      counts[c.status]++;
    }
    return counts;
  });

  // Methods
  function getCurrentAgentConfig() {
    const store = useSettingsStore();
    return store.agents.find((a) => a.id === currentAgentId.value);
  }

  function addProject(path: string) {
    if (!projectPaths.value.includes(path)) {
      projectPaths.value.push(path);
      saveProjectPaths(projectPaths.value);
    }
    if (!currentProjectPath.value) {
      currentProjectPath.value = path;
    }
  }

  function removeProject(path: string) {
    projectPaths.value = projectPaths.value.filter((p) => p !== path);
    saveProjectPaths(projectPaths.value);
    if (currentProjectPath.value === path) {
      currentProjectPath.value = projectPaths.value[0] ?? "";
    }
  }

  async function scanSkills(agentId: string, scope: SkillScope): Promise<void> {
    const store = useSettingsStore();
    const agent = store.agents.find((a) => a.id === agentId);
    if (!agent) {
      throw new Error(`Agent ${agentId} not found in settings`);
    }

    loading.value = true;
    error.value = null;

    try {
      let result: ScanResult;

      if (scope === "global") {
        result = await invoke<ScanResult>("scan_global_skills", {
          agentId,
          globalPath: agent.globalPath + "/skills",
        });
        globalSkills.value.set(agentId, result);
      } else {
        // Project skills: need a selected project
        if (!currentProjectPath.value) {
          // No project selected → empty result
          projectSkills.value.set(agentId, {
            agentId,
            scope: "project",
            skills: [],
            total: 0,
          });
          loading.value = false;
          return;
        }
        result = await invoke<ScanResult>("scan_project_skills", {
          agentId,
          projectPath: agent.projectPath + "/skills",
          projectDir: currentProjectPath.value,
        });
        projectSkills.value.set(agentId, result);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      console.error(`[SkillsStore] Failed to scan ${scope} skills for ${agentId}:`, e);
    } finally {
      loading.value = false;
    }
  }

  async function switchAgent(agentId: string): Promise<void> {
    currentAgentId.value = agentId;
    selectedSkills.value.clear();
    comparisons.value = [];

    const skillsMap =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    if (!skillsMap.has(agentId)) {
      await scanSkills(agentId, currentScope.value);
    }
    // Always reload comparisons — views depend on them
    await loadComparisons();
  }

  async function switchScope(scope: SkillScope): Promise<void> {
    currentScope.value = scope;
    selectedSkills.value.clear();
    comparisons.value = [];

    const skillsMap =
      scope === "global" ? globalSkills.value : projectSkills.value;
    if (!skillsMap.has(currentAgentId.value)) {
      await scanSkills(currentAgentId.value, scope);
    }
    if (comparisonMode.value) {
      await loadComparisons();
    }
  }

  async function selectProject(path: string): Promise<void> {
    currentProjectPath.value = path;
    // Clear cached project skills and rescan
    projectSkills.value.clear();
    if (currentScope.value === "project") {
      await scanSkills(currentAgentId.value, "project");
    }
  }

  async function installSkill(source: string, targetPath: string): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      await invoke<OperationResult>("install_skill", { source, targetPath });
      await scanSkills(currentAgentId.value, currentScope.value);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateSkill(source: string, targetPath: string): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      await invoke<OperationResult>("update_skill", { source, targetPath });
      await scanSkills(currentAgentId.value, currentScope.value);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function uninstallSkill(skillPath: string): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      await invoke<OperationResult>("uninstall_skill", { skillPath });
      await scanSkills(currentAgentId.value, currentScope.value);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function batchOperate(operations: SkillOperation[]): Promise<OperationResult[]> {
    loading.value = true;
    error.value = null;

    try {
      const results = await invoke<OperationResult[]>("batch_operate", { operations });
      await scanSkills(currentAgentId.value, currentScope.value);
      return results;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  function toggleSelect(skillName: string): void {
    if (selectedSkills.value.has(skillName)) {
      selectedSkills.value.delete(skillName);
    } else {
      selectedSkills.value.add(skillName);
    }
  }

  function selectAll(): void {
    currentSkills.value.forEach((skill) => {
      selectedSkills.value.add(skill.name);
    });
  }

  function clearSelection(): void {
    selectedSkills.value.clear();
  }

  /** Resolve the local skill directory for a given scope + project */
  function resolveLocalDir(scope: SkillScope, projectPath?: string): string {
    const agent = getCurrentAgentConfig();
    if (!agent) return "";
    if (scope === "global") {
      return agent.globalPath + "/skills";
    }
    // Project: projectPath is the project root (e.g. D:\Project\my-app)
    const dir = projectPath ?? currentProjectPath.value;
    if (!dir) return "";
    return dir + "/" + agent.projectPath + "/skills";
  }

  async function loadComparisons(scope?: SkillScope, projectPath?: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const settingsStore = useSettingsStore();
      const resolvedScope = scope ?? currentScope.value;
      const localDir = resolveLocalDir(resolvedScope, projectPath);
      if (!localDir) {
        comparisons.value = [];
        loading.value = false;
        return;
      }

      comparisons.value = await invoke<SkillComparison[]>("compare_skills", {
        localDir,
        repos: settingsStore.repos,
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      console.error("[SkillsStore] Failed to load comparisons:", e);
    } finally {
      loading.value = false;
    }
  }

  function toggleComparisonMode(): void {
    comparisonMode.value = !comparisonMode.value;
    if (comparisonMode.value) {
      loadComparisons();
    }
  }

  async function loadSkillDiff(
    localPath: string,
    remotePath: string,
  ): Promise<SkillDiffResult> {
    loadingDiff.value = true;
    try {
      const result = await invoke<SkillDiffResult>("get_skill_diff", {
        localSkillPath: localPath,
        remoteSkillPath: remotePath,
      });
      currentDiff.value = result;
      return result;
    } finally {
      loadingDiff.value = false;
    }
  }

  async function loadDiffFileContent(
    localSkillPath: string,
    remoteSkillPath: string,
    relFilePath: string,
  ): Promise<DiffFileContent> {
    return invoke<DiffFileContent>("get_diff_file_content", {
      localSkillPath,
      remoteSkillPath,
      relFilePath,
    });
  }

  function clearDiff(): void {
    currentDiff.value = null;
  }

  async function getOperationHistory(limit?: number) {
    operationHistory.value = await invoke<OperationRecord[]>('get_operation_history', { limit });
  }

  async function rollbackOperation(id: string) {
    await invoke<string>('rollback_operation', { id });
    await getOperationHistory();
    await loadComparisons();
  }

  async function clearHistory() {
    await invoke('clear_history');
    operationHistory.value = [];
  }

  return {
    // State
    currentAgentId,
    currentScope,
    currentProjectPath,
    projectPaths,
    globalSkills,
    projectSkills,
    selectedSkills,
    loading,
    error,

    // Computed
    currentSkills,
    availableAgents,

    // Methods
    getCurrentAgentConfig,
    resolveLocalDir,
    scanSkills,
    switchAgent,
    switchScope,
    selectProject,
    addProject,
    removeProject,
    installSkill,
    updateSkill,
    uninstallSkill,
    batchOperate,
    toggleSelect,
    selectAll,
    clearSelection,

    // Comparison
    comparisons,
    comparisonMode,
    comparisonCounts,
    currentDiff,
    loadingDiff,
    loadComparisons,
    toggleComparisonMode,
    loadSkillDiff,
    loadDiffFileContent,
    clearDiff,

    // History
    operationHistory,
    showHistoryPanel,
    getOperationHistory,
    rollbackOperation,
    clearHistory,
  };
});
