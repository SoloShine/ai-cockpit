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
} from "./types";

export const useSkillsStore = defineStore("skills", () => {
  // State
  const currentAgentId = ref<string>("claude-code");
  const currentScope = ref<SkillScope>("global");
  const currentProjectPath = ref<string>("");
  const globalSkills = ref<Map<string, ScanResult>>(new Map());
  const projectSkills = ref<Map<string, ScanResult>>(new Map());
  const selectedSkills = ref<Set<string>>(new Set());
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const currentSkills = computed<SkillInfo[]>(() => {
    const store = useSettingsStore();
    const skillsMap =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    const result = skillsMap.get(currentAgentId.value);
    return result?.skills ?? [];
  });

  const availableAgents = computed(() => {
    const store = useSettingsStore();
    return store.agents.filter((a) => a.enabled);
  });

  // Methods
  function getCurrentAgentConfig() {
    const store = useSettingsStore();
    return store.agents.find((a) => a.id === currentAgentId.value);
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
          globalPath: agent.globalPath,
        });
        globalSkills.value.set(agentId, result);
      } else {
        result = await invoke<ScanResult>("scan_project_skills", {
          agentId,
          projectPath: agent.projectPath,
          projectDir: currentProjectPath.value || ".",
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

  async function scanAllAgents(scope: SkillScope): Promise<void> {
    const agents = availableAgents.value;
    if (agents.length === 0) return;

    loading.value = true;
    error.value = null;

    try {
      await Promise.all(
        agents.map((agent) => scanSkills(agent.id, scope))
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      console.error("[SkillsStore] Failed to scan all agents:", e);
    } finally {
      loading.value = false;
    }
  }

  async function switchAgent(agentId: string): Promise<void> {
    currentAgentId.value = agentId;
    selectedSkills.value.clear();

    // Check if we already have cached data
    const skillsMap =
      currentScope.value === "global" ? globalSkills.value : projectSkills.value;
    if (!skillsMap.has(agentId)) {
      await scanSkills(agentId, currentScope.value);
    }
  }

  async function switchScope(scope: SkillScope): Promise<void> {
    currentScope.value = scope;
    selectedSkills.value.clear();
    await scanAllAgents(scope);
  }

  async function installSkill(source: string, targetPath: string): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      await invoke<OperationResult>("install_skill", {
        source,
        targetPath,
      });
      // Rescan after operation
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
      await invoke<OperationResult>("update_skill", {
        source,
        targetPath,
      });
      // Rescan after operation
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
      await invoke<OperationResult>("uninstall_skill", {
        skillPath,
      });
      // Rescan after operation
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
      const results = await invoke<OperationResult[]>("batch_operate", {
        operations,
      });
      // Rescan after operation
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

  return {
    // State
    currentAgentId,
    currentScope,
    currentProjectPath,
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
    scanSkills,
    scanAllAgents,
    switchAgent,
    switchScope,
    installSkill,
    updateSkill,
    uninstallSkill,
    batchOperate,
    toggleSelect,
    selectAll,
    clearSelection,
  };
});
