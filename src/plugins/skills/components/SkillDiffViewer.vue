<template>
  <n-modal
    :show="props.show"
    :mask-closable="true"
    @update:show="(v) => !v && emit('close')"
  >
    <n-card
      style="width: 90vw; max-width: 1000px"
      :title="t('diff.title', { name: diffResult?.skillName || '' })"
      size="small"
      closable
      @close="emit('close')"
    >
      <template #header-extra>
        <n-space v-if="diffResult" :size="8">
          <n-tag v-if="diffResult.addedCount > 0" type="success" size="small">
            {{ t('diff.added') }}: {{ diffResult.addedCount }}
          </n-tag>
          <n-tag v-if="diffResult.removedCount > 0" type="error" size="small">
            {{ t('diff.removed') }}: {{ diffResult.removedCount }}
          </n-tag>
          <n-tag v-if="diffResult.modifiedCount > 0" type="warning" size="small">
            {{ t('diff.modified') }}: {{ diffResult.modifiedCount }}
          </n-tag>
          <n-tag v-if="diffResult.unchangedCount > 0" size="small">
            {{ t('diff.unchanged') }}: {{ diffResult.unchangedCount }}
          </n-tag>
        </n-space>
      </template>

      <n-spin :show="loadingDiff || loadingFileContent">
        <div style="display: flex; gap: 16px; height: 500px">
          <!-- Left panel: file list -->
          <div style="width: 280px; overflow: hidden; display: flex; flex-direction: column">
            <n-data-table
              :columns="fileColumns"
              :data="diffResult?.fileDiffs || []"
              :row-props="fileRowProps"
              :max-height="460"
              size="small"
              :bordered="false"
            />
          </div>

          <!-- Right panel: line diff -->
          <div style="flex: 1; overflow: hidden; display: flex; flex-direction: column">
            <div v-if="selectedFile" style="margin-bottom: 8px; font-weight: 500">
              {{ selectedFile.fileName }}
            </div>
            <div
              v-if="fileDiffLines.length > 0"
              style="flex: 1; overflow: auto; border: 1px solid #e0e0e6; border-radius: 4px"
            >
              <table class="diff-table">
                <tbody>
                  <tr
                    v-for="(line, idx) in fileDiffLines"
                    :key="idx"
                    :class="`diff-line diff-line-${line.type}`"
                  >
                    <td class="diff-line-number">{{ line.oldLineNumber || '' }}</td>
                    <td class="diff-line-number">{{ line.newLineNumber || '' }}</td>
                    <td class="diff-prefix">{{ getPrefix(line.type) }}</td>
                    <td class="diff-content">{{ line.content }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else style="flex: 1; display: flex; align-items: center; justify-content: center; color: #999">
              {{ t('diff.selectFile') }}
            </div>
          </div>
        </div>
      </n-spin>
    </n-card>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch, h } from "vue";
import { NModal, NCard, NSpace, NTag, NSpin, NDataTable, type DataTableColumns } from "naive-ui";
import { useI18n } from "vue-i18n";
import { diffLines } from "diff";
import { useSkillsStore } from "../store";
import type { FileDiffEntry, DiffLine, DiffStatus, SkillDiffResult } from "../types";

const props = defineProps<{
  show: boolean;
  localPath: string;
  remotePath: string;
}>();

const emit = defineEmits<{ close: [] }>();

const { t } = useI18n();
const store = useSkillsStore();

const diffResult = ref<SkillDiffResult | null>(null);
const selectedFile = ref<FileDiffEntry | null>(null);
const fileDiffLines = ref<DiffLine[]>([]);
const loadingDiff = ref(false);
const loadingFileContent = ref(false);

// Get prefix for diff line
function getPrefix(type: DiffLine["type"]): string {
  switch (type) {
    case "added":
      return "+";
    case "removed":
      return "-";
    case "unchanged":
      return " ";
  }
}

// Get status tag type
function getStatusType(status: DiffStatus) {
  switch (status) {
    case "added":
      return "success";
    case "removed":
      return "error";
    case "modified":
      return "warning";
    case "same":
      return "default";
  }
}

// Get status label
function getStatusLabel(status: DiffStatus): string {
  return t(`diff.${status}`);
}

// File table columns
const fileColumns: DataTableColumns<FileDiffEntry> = [
  {
    key: "fileName",
    title: t("diff.file"),
    width: 120,
    ellipsis: { tooltip: true },
  },
  {
    key: "status",
    title: t("diff.status"),
    width: 80,
    render: (row) => {
      return h(
        NTag,
        { type: getStatusType(row.diffType), size: "small", bordered: false },
        { default: () => getStatusLabel(row.diffType) }
      );
    },
  },
];

// File row props - enable click to select
const fileRowProps = (row: FileDiffEntry) => {
  return {
    style: {
      cursor: row.diffType === "same" ? "default" : "pointer",
      opacity: row.diffType === "same" ? 0.5 : 1,
    },
    onClick: () => {
      if (row.diffType !== "same") {
        loadFileDiff(row);
      }
    },
  };
};

// Load file diff content
async function loadFileDiff(fileEntry: FileDiffEntry) {
  selectedFile.value = fileEntry;
  loadingFileContent.value = true;
  fileDiffLines.value = [];

  try {
    const content = await store.loadDiffFileContent(
      props.localPath,
      props.remotePath,
      fileEntry.path
    );

    const remoteContent = content.remoteContent || "";
    const localContent = content.localContent || "";

    // Compute diff
    const changes = diffLines(remoteContent, localContent);

    const lines: DiffLine[] = [];
    let oldLine = 1;
    let newLine = 1;

    for (const change of changes) {
      const count = change.count || 1;
      const content = change.value;

      if (change.added) {
        for (let i = 0; i < count; i++) {
          const lineContent = content.split("\n")[i] || "";
          if (lineContent || i < count - 1) {
            lines.push({
              type: "added",
              oldLineNumber: undefined,
              newLineNumber: newLine++,
              content: lineContent,
            });
          }
        }
      } else if (change.removed) {
        for (let i = 0; i < count; i++) {
          const lineContent = content.split("\n")[i] || "";
          if (lineContent || i < count - 1) {
            lines.push({
              type: "removed",
              oldLineNumber: oldLine++,
              newLineNumber: undefined,
              content: lineContent,
            });
          }
        }
      } else {
        for (let i = 0; i < count; i++) {
          const lineContent = content.split("\n")[i] || "";
          if (lineContent || i < count - 1) {
            lines.push({
              type: "unchanged",
              oldLineNumber: oldLine++,
              newLineNumber: newLine++,
              content: lineContent,
            });
          }
        }
      }
    }

    fileDiffLines.value = lines;
  } catch (e) {
    console.error("[SkillDiffViewer] Failed to load file diff:", e);
  } finally {
    loadingFileContent.value = false;
  }
}

// Watch for modal open
watch(
  () => props.show,
  async (show) => {
    if (show && props.localPath && props.remotePath) {
      loadingDiff.value = true;
      selectedFile.value = null;
      fileDiffLines.value = [];

      try {
        diffResult.value = await store.loadSkillDiff(props.localPath, props.remotePath);
      } catch (e) {
        console.error("[SkillDiffViewer] Failed to load skill diff:", e);
      } finally {
        loadingDiff.value = false;
      }
    }
  }
);
</script>

<style scoped>
.diff-table {
  width: 100%;
  border-collapse: collapse;
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  font-size: 13px;
  background: white;
}

.diff-line {
  border-bottom: 1px solid #f0f0f0;
}

.diff-line-added {
  background-color: rgba(46, 160, 67, 0.15);
}

.diff-line-removed {
  background-color: rgba(248, 81, 73, 0.15);
}

.diff-line-unchanged {
  background-color: transparent;
}

.diff-line-number {
  width: 40px;
  padding: 2px 8px;
  text-align: right;
  color: #999;
  user-select: none;
  border-right: 1px solid #e0e0e6;
}

.diff-prefix {
  width: 20px;
  padding: 2px 4px;
  text-align: center;
  font-weight: bold;
  user-select: none;
  border-right: 1px solid #e0e0e6;
}

.diff-content {
  padding: 2px 8px;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
