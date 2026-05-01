<script setup lang="ts">
import { ref, computed, h } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import {
  NDataTable,
  type DataTableColumns,
  NSpace,
  NButton,
  NTag,
  NTooltip,
  useDialog,
  type DataTableRowKey,
} from "naive-ui";
import { useSkillsStore } from "../store";
import type { SkillComparison, SkillOperation, ComparisonStatus } from "../types";

const emit = defineEmits<{
  diff: [localPath: string, remotePath: string];
  preview: [skillPath: string, skillName: string];
}>();

const { t } = useI18n();
const dialog = useDialog();
const store = useSkillsStore();

const checkedRowKeys = ref<DataTableRowKey[]>([]);
const operatingKeys = ref<Set<string>>(new Set());

// Helper to get home directory
async function getHome(): Promise<string> {
  const dataDir = await invoke<string>("get_data_dir");
  return dataDir + "/..";
}

// Get status tag type and color
function getStatusTagType(status: ComparisonStatus): {
  type: "success" | "warning" | "info" | "error" | "default";
  label: string;
} {
  switch (status) {
    case "same":
      return { type: "success", label: t("skills.compare.statusSame") };
    case "outdated":
      return { type: "warning", label: t("skills.compare.statusOutdated") };
    case "localOnly":
      return { type: "info", label: t("skills.compare.statusLocalOnly") };
    case "remoteOnly":
      return { type: "error", label: t("skills.compare.statusRemoteOnly") };
    default:
      return { type: "default", label: status };
  }
}

// Table columns
const columns = computed<DataTableColumns<SkillComparison>>(() => [
  {
    type: "selection",
    multiple: true,
  },
  {
    title: t("skills.compare.status"),
    key: "status",
    width: 120,
    render: (row) => {
      const { type, label } = getStatusTagType(row.status);
      return h(NTag, { type }, { default: () => label });
    },
  },
  {
    title: t("skills.compare.name"),
    key: "name",
    width: 200,
    render: (row) => {
      const meta = row.local?.meta || row.remote?.meta;
      return meta?.name || row.name;
    },
  },
  {
    title: t("skills.compare.sourceRepo"),
    key: "sourceRepo",
    width: 150,
    render: (row) => row.sourceRepo || "-",
  },
  {
    title: t("skills.compare.localVersion"),
    key: "localVersion",
    width: 100,
    render: (row) => row.local?.meta?.version || "-",
  },
  {
    title: t("skills.compare.remoteVersion"),
    key: "remoteVersion",
    width: 100,
    render: (row) => row.remote?.meta?.version || "-",
  },
  {
    title: t("skills.compare.description"),
    key: "description",
    ellipsis: {
      tooltip: true,
    },
    render: (row) => {
      const meta = row.local?.meta || row.remote?.meta;
      return meta?.description || "-";
    },
  },
  {
    title: t("skills.compare.actions"),
    key: "actions",
    width: 180,
    fixed: "right" as const,
    render: (row) => {
      const buttons: ReturnType<typeof h>[] = [];

      // Install button for remoteOnly
      if (row.status === "remoteOnly" && row.remote) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              onClick: () => handleInstall(row),
              loading: operatingKeys.value.has(row.name),
            },
            { default: () => t("skills.actions.install") }
          )
        );
      }

      // Diff button for outdated
      if (row.status === "outdated" && row.local && row.remote) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              onClick: () => emit("diff", row.local!.path, row.remote!.path),
            },
            { default: () => t("skills.compare.viewDiff") }
          )
        );
      }

      // Update button for outdated
      if (row.status === "outdated" && row.local && row.remote) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              type: "primary",
              onClick: () => handleUpdate(row),
              loading: operatingKeys.value.has(row.name),
            },
            { default: () => t("skills.actions.update") }
          )
        );
      }

      // Reinstall button for same
      if (row.status === "same" && row.local && row.remote) {
        buttons.push(
          h(
            NTooltip,
            {},
            {
              trigger: () =>
                h(
                  NButton,
                  {
                    size: "small",
                    onClick: () => handleUpdate(row),
                    loading: operatingKeys.value.has(row.name),
                  },
                  { default: () => t("skills.compare.reinstall") }
                ),
              default: () => t("skills.compare.reinstallTip"),
            }
          )
        );
      }

      // Uninstall button for localOnly
      if (row.status === "localOnly" && row.local) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              type: "error",
              onClick: () => handleUninstall(row),
              loading: operatingKeys.value.has(row.name),
            },
            { default: () => t("skills.actions.uninstall") }
          )
        );
      }

      // Preview button for any row with remote
      if (row.remote) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              onClick: () => emit("preview", row.remote!.path, row.name),
            },
            { default: () => t("skills.compare.preview") }
          )
        );
      }

      return h(NSpace, {}, { default: () => buttons });
    },
  },
]);

// Action handlers
async function handleInstall(row: SkillComparison) {
  if (!row.remote || !row.remote?.path) return;

  operatingKeys.value.add(row.name);
  try {
    const agentConfig = store.getCurrentAgentConfig();
    if (!agentConfig) return;

    const home = await getHome();
    const globalPath = agentConfig.globalPath.replace("~", home);
    const targetPath = `${globalPath}/skills/${row.name}`;

    await store.installSkill(row.remote.path, targetPath);
    await store.loadComparisons();
  } finally {
    operatingKeys.value.delete(row.name);
  }
}

async function handleUpdate(row: SkillComparison) {
  if (!row.local?.path || !row.remote?.path) return;

  operatingKeys.value.add(row.name);
  try {
    await store.updateSkill(row.remote.path, row.local.path);
    await store.loadComparisons();
  } finally {
    operatingKeys.value.delete(row.name);
  }
}

function handleUninstall(row: SkillComparison) {
  if (!row.local?.path) return;

  dialog.warning({
    title: t("skills.confirm.uninstall"),
    content: t("skills.confirm.uninstallMsg", { name: row.name }),
    positiveText: t("skills.actions.uninstall"),
    negativeText: t("skills.confirm.cancel"),
    onPositiveClick: async () => {
      operatingKeys.value.add(row.name);
      try {
        await store.uninstallSkill(row.local!.path);
        await store.loadComparisons();
      } finally {
        operatingKeys.value.delete(row.name);
      }
    },
  });
}

// Batch operations
const canBatchInstall = computed(() => {
  return checkedRowKeys.value.some((key) => {
    const row = store.comparisons.find((r) => r.name === key);
    return row?.status === "remoteOnly";
  });
});

const canBatchUpdate = computed(() => {
  return checkedRowKeys.value.some((key) => {
    const row = store.comparisons.find((r) => r.name === key);
    return row?.status === "outdated" || row?.status === "same";
  });
});

async function handleBatchInstall() {
  const operations: SkillOperation[] = [];

  for (const key of checkedRowKeys.value) {
    const row = store.comparisons.find((r) => r.name === key);
    if (row?.status === "remoteOnly" && row.remote?.path) {
      const agentConfig = store.getCurrentAgentConfig();
      if (!agentConfig) continue;

      const home = await getHome();
      const globalPath = agentConfig.globalPath.replace("~", home);
      const targetPath = `${globalPath}/skills/${row.name}`;

      operations.push({
        operationType: "install",
        source: row.remote.path,
        targetPath,
      });
    }
  }

  if (operations.length > 0) {
    await store.batchOperate(operations);
    await store.loadComparisons();
    checkedRowKeys.value = [];
  }
}

async function handleBatchUpdate() {
  const operations: SkillOperation[] = [];

  for (const key of checkedRowKeys.value) {
    const row = store.comparisons.find((r) => r.name === key);
    if (
      (row?.status === "outdated" || row?.status === "same") &&
      row.local?.path &&
      row.remote?.path
    ) {
      operations.push({
        operationType: "update",
        source: row.remote.path,
        targetPath: row.local.path,
      });
    }
  }

  if (operations.length > 0) {
    await store.batchOperate(operations);
    await store.loadComparisons();
    checkedRowKeys.value = [];
  }
}
</script>

<template>
  <div class="skill-compare-table">
    <n-space v-if="canBatchInstall || canBatchUpdate" :mb="3">
      <n-button
        v-if="canBatchInstall"
        type="primary"
        :disabled="checkedRowKeys.length === 0"
        @click="handleBatchInstall"
      >
        {{ t("skills.compare.batchInstall") }}
      </n-button>
      <n-button
        v-if="canBatchUpdate"
        type="primary"
        :disabled="checkedRowKeys.length === 0"
        @click="handleBatchUpdate"
      >
        {{ t("skills.compare.batchUpdate") }}
      </n-button>
    </n-space>

    <n-data-table
      :columns="columns"
      :data="store.comparisons"
      :row-key="(row: SkillComparison) => row.name"
      :checked-row-keys="checkedRowKeys"
      :max-height="600"
      :loading="store.loading"
      @update:checked-row-keys="
        (keys: DataTableRowKey[]) => (checkedRowKeys = keys)
      "
    />
  </div>
</template>

<style scoped>
.skill-compare-table {
  width: 100%;
}
</style>
