<script setup lang="ts">
import { ref, computed, h } from "vue";
import { useI18n } from "vue-i18n";
import {
  NDataTable,
  type DataTableColumns,
  NSpace,
  NButton,
  NTag,
  NTooltip,
  NText,
  NAlert,
  NEmpty,
  useDialog,
  type DataTableRowKey,
} from "naive-ui";
import { useSkillsStore } from "../store";
import { useSettingsStore } from "@/plugins/settings/store";
import { useColumnResize } from "../composables/useColumnResize";
import type { SkillComparison, SkillOperation, ComparisonStatus } from "../types";

const emit = defineEmits<{
  diff: [localPath: string, remotePath: string];
  preview: [skillPath: string, skillName: string];
}>();

const { t } = useI18n();
const dialog = useDialog();
const store = useSkillsStore();
const settingsStore = useSettingsStore();

const checkedRowKeys = ref<DataTableRowKey[]>([]);
const operatingKeys = ref<Set<string>>(new Set());

// Column resize support
const { getColumnWidth, handleResizeMousedown } = useColumnResize(
  "skill-compare-table",
  { status: 90, name: 160, sourceRepo: 120, localVersion: 80, remoteVersion: 80, actions: 150 },
);

/** Render a resizable column header with drag handle */
function resizableHeader(key: string, titleText: string) {
  return h(
    "div",
    {
      style: "position: relative; padding-right: 10px; display: flex; align-items: center; justify-content: space-between; width: 100%",
    },
    [
      h("span", titleText),
      h("div", {
        class: "col-resize-handle",
        onMousedown: (e: MouseEvent) => handleResizeMousedown(key, e),
      }),
    ],
  );
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
    title: () => resizableHeader("status", t("skills.compare.status")),
    key: "status",
    width: getColumnWidth("status"),
    render: (row) => {
      const { type, label } = getStatusTagType(row.status);
      return h(NTag, { type }, { default: () => label });
    },
  },
  {
    title: () => resizableHeader("name", t("skills.compare.name")),
    key: "name",
    width: getColumnWidth("name"),
    render: (row) => {
      const meta = row.local?.meta || row.remote?.meta;
      return meta?.name || row.name;
    },
  },
  {
    title: () => resizableHeader("sourceRepo", t("skills.compare.sourceRepo")),
    key: "sourceRepo",
    width: getColumnWidth("sourceRepo"),
    render: (row) => {
      const repoId = row.sourceRepo;
      if (!repoId) return h(NText, { depth: 3 }, () => "-");
      const repo = settingsStore.repos.find((r) => r.id === repoId);
      const name = repo?.name || repoId;
      return h(NTag, { size: "small", type: "info", round: true }, { default: () => name });
    },
  },
  {
    title: () => resizableHeader("localVersion", t("skills.compare.localVersion")),
    key: "localVersion",
    width: getColumnWidth("localVersion"),
    render: (row) => row.local?.meta?.version || "-",
  },
  {
    title: () => resizableHeader("remoteVersion", t("skills.compare.remoteVersion")),
    key: "remoteVersion",
    width: getColumnWidth("remoteVersion"),
    render: (row) => row.remote?.meta?.version || "-",
  },
  {
    title: t("skills.compare.description"),
    key: "description",
    minWidth: 200,
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

      // Uninstall button for any skill that exists locally
      if (row.local) {
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

      // Preview button — prefer local if available, otherwise remote
      const previewPath = row.local?.path || row.remote?.path;
      if (previewPath) {
        buttons.push(
          h(
            NButton,
            {
              size: "small",
              onClick: () => emit("preview", previewPath, row.name),
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
    const localDir = store.resolveLocalDir(store.currentScope);
    if (!localDir) return;
    const targetPath = `${localDir}/${row.name}`;

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
  const localDir = store.resolveLocalDir(store.currentScope);
  if (!localDir) return;

  for (const key of checkedRowKeys.value) {
    const row = store.comparisons.find((r) => r.name === key);
    if (row?.status === "remoteOnly" && row.remote?.path) {
      operations.push({
        operationType: "install",
        source: row.remote.path,
        targetPath: `${localDir}/${row.name}`,
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
    <n-alert v-if="store.error" type="error" :title="t('skills.compare.errorTitle')" style="margin-bottom: 12px">
      {{ store.error }}
    </n-alert>

    <n-empty
      v-if="!store.loading && store.filteredComparisons.length === 0 && !store.error"
      :description="store.searchText || store.statusFilter ? t('skills.filter.noResults') : t('skills.compare.empty')"
      style="padding: 40px 0"
    />

    <template v-if="store.filteredComparisons.length > 0 || store.loading">
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
      :data="store.filteredComparisons"
      :row-key="(row: SkillComparison) => row.name"
      :checked-row-keys="checkedRowKeys"
      :max-height="600"
      :loading="store.loading"
      @update:checked-row-keys="
        (keys: DataTableRowKey[]) => (checkedRowKeys = keys)
      "
    />
    </template>
  </div>
</template>

<style scoped>
.skill-compare-table {
  width: 100%;
}
</style>

<style>
/* Global styles for resize handles (scoped styles won't reach NDataTable internals) */
.col-resize-handle {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  z-index: 1;
  transition: background 0.15s;
}
.col-resize-handle:hover {
  background: rgba(0, 0, 0, 0.12);
}
</style>
