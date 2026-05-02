<script setup lang="ts">
import { NModal, NButton, NSpace, NText, NDataTable, NSpin } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { computed, h, ref, watch } from 'vue'
import * as Diff from 'diff'
import { useI18n } from 'vue-i18n'
import type { FileDiffEntry, SkillDiffResult } from '../types'

const { t } = useI18n()

const props = defineProps<{
  show: boolean
  localPath: string
  remotePath: string
}>()

const emit = defineEmits<{
  close: []
}>()

const diffResult = ref<SkillDiffResult | null>(null)
const selectedFile = ref<string | null>(null)
const loadingDiff = ref(false)
const loadingFileContent = ref(false)
const diffLines = ref<DiffLine[]>([])

interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  oldLineNo?: number
  newLineNo?: number
  content: string
}

const statusIcon = (status: FileDiffEntry['diffType']) => {
  const map: Record<string, { color: string; label: string }> = {
    same: { color: '#94a3b8', label: t('skills.diff.unchanged') },
    added: { color: '#10b981', label: t('skills.diff.added') },
    removed: { color: '#ef4444', label: t('skills.diff.removed') },
    modified: { color: '#f59e0b', label: t('skills.diff.modified') },
  }
  const info = map[status]
  return h(NSpace, { size: 4, align: 'center' }, () => [
    h('span', { style: `display:inline-block;width:8px;height:8px;border-radius:50%;background:${info.color}` }),
    h(NText, { style: 'font-size: 12px' }, () => info.label),
  ])
}

function formatSize(size?: number): string {
  if (size === undefined || size === null) return '-'
  if (size < 1024) return `${size} B`
  return `${(size / 1024).toFixed(1)} KB`
}

async function loadSkillDiff() {
  if (!props.localPath || !props.remotePath) return

  loadingDiff.value = true
  selectedFile.value = null
  diffLines.value = []
  diffResult.value = null

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    diffResult.value = await invoke<SkillDiffResult>('get_skill_diff', {
      localSkillPath: props.localPath,
      remoteSkillPath: props.remotePath,
    })
  } catch (e) {
    console.error('[SkillDiffViewer] Failed to load skill diff:', e)
  } finally {
    loadingDiff.value = false
  }
}

async function handleFileClick(row: FileDiffEntry) {
  if (row.diffType === 'same') return
  if (selectedFile.value === row.path) {
    selectedFile.value = null
    diffLines.value = []
    return
  }

  selectedFile.value = row.path
  loadingFileContent.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke<{ localContent?: string; remoteContent?: string }>('get_diff_file_content', {
      localSkillPath: props.localPath,
      remoteSkillPath: props.remotePath,
      relFilePath: row.path,
    })
    computeDiffLines(result)
  } catch (e) {
    console.error('[SkillDiffViewer] Failed to load file diff:', e)
    diffLines.value = []
  } finally {
    loadingFileContent.value = false
  }
}

function computeDiffLines(result: { localContent?: string; remoteContent?: string }) {
  // Normalize line endings to avoid \r\n vs \n causing entire file to diff
  const local = (result.localContent ?? '').replace(/\r\n/g, '\n')
  const remote = (result.remoteContent ?? '').replace(/\r\n/g, '\n')
  // diffLines(oldStr, newStr): added = in new, removed = in old
  // Show "what changed from local to remote" — remote is the update source
  const changes = Diff.diffLines(local, remote)

  const lines: DiffLine[] = []
  let oldLine = 1
  let newLine = 1

  for (const change of changes) {
    const textLines = change.value.replace(/\n$/, '').split('\n')

    if (change.added) {
      // Lines in remote (new) not in local — update additions
      for (const line of textLines) {
        lines.push({ type: 'added', newLineNo: newLine++, content: line })
      }
    } else if (change.removed) {
      // Lines in local (old) not in remote — will be removed by update
      for (const line of textLines) {
        lines.push({ type: 'removed', oldLineNo: oldLine++, content: line })
      }
    } else {
      for (const line of textLines) {
        lines.push({ type: 'unchanged', oldLineNo: oldLine++, newLineNo: newLine++, content: line })
      }
    }
  }

  diffLines.value = lines
}

const changedFiles = computed(() =>
  diffResult.value?.fileDiffs.filter(f => f.diffType !== 'same') ?? []
)

const columns = computed<DataTableColumns<FileDiffEntry>>(() => [
  {
    title: t('skills.diff.status'),
    key: 'status',
    width: 100,
    render: (row) => statusIcon(row.diffType),
    sorter: (a, b) => a.diffType.localeCompare(b.diffType),
  },
  {
    title: t('skills.diff.file'),
    key: 'path',
    render: (row) =>
      h(NText, {
        style: `font-family: monospace; font-size: 13px; cursor: pointer; ${selectedFile.value === row.path ? 'color: #18a058; font-weight: 600;' : ''}`,
      }, () => row.path),
  },
  {
    title: t('skills.diff.sizeDelta'),
    key: 'sizeDelta',
    width: 100,
    render: (row) => {
      if (row.diffType === 'added')
        return h(NText, { style: 'color: #10b981; font-size: 12px' }, () => `+${formatSize(row.remoteSize)}`)
      if (row.diffType === 'removed')
        return h(NText, { style: 'color: #ef4444; font-size: 12px' }, () => `-${formatSize(row.localSize)}`)
      if (row.diffType === 'modified') {
        const delta = (row.remoteSize ?? 0) - (row.localSize ?? 0)
        const sign = delta >= 0 ? '+' : ''
        const color = delta >= 0 ? '#10b981' : '#ef4444'
        return h(NText, { style: `color: ${color}; font-size: 12px` }, () => `${sign}${formatSize(Math.abs(delta))}`)
      }
      return h(NText, { depth: 3, style: 'font-size: 12px' }, () => '-')
    },
  },
])

const addedLines = computed(() => diffLines.value.filter((l) => l.type === 'added').length)
const removedLines = computed(() => diffLines.value.filter((l) => l.type === 'removed').length)

// Watch for modal open
watch(() => props.show, (show) => {
  if (show) {
    loadSkillDiff()
  }
})
</script>

<template>
  <NModal :show="props.show" @update:show="(v) => !v && emit('close')">
    <div class="diff-dialog">
      <!-- Header -->
      <div class="diff-header">
        <span class="diff-title">{{ t('skills.diff.title') }} — {{ diffResult?.skillName || '' }}</span>
        <NButton quaternary size="small" @click="emit('close')">{{ t('skills.diff.close') }}</NButton>
      </div>

      <!-- Scrollable body -->
      <div class="diff-scroll">
        <NSpin :show="loadingDiff">
          <div v-if="diffResult">
            <!-- Summary bar -->
            <div class="diff-summary">
              <NSpace size="small">
                <span v-if="diffResult.addedCount > 0" class="diff-badge diff-added">
                  {{ t('skills.diff.added') }} {{ diffResult.addedCount }}
                </span>
                <span v-if="diffResult.removedCount > 0" class="diff-badge diff-removed">
                  {{ t('skills.diff.removed') }} {{ diffResult.removedCount }}
                </span>
                <span v-if="diffResult.modifiedCount > 0" class="diff-badge diff-modified">
                  {{ t('skills.diff.modified') }} {{ diffResult.modifiedCount }}
                </span>
              </NSpace>
            </div>

            <!-- File list table -->
            <NDataTable
              :columns="columns"
              :data="changedFiles"
              :bordered="false"
              size="small"
              :max-height="selectedFile ? 180 : 300"
              virtual-scroll
              :row-props="(row: FileDiffEntry) => ({ style: 'cursor: pointer', onClick: () => handleFileClick(row) })"
            />

            <!-- File content diff panel -->
            <div v-if="selectedFile" class="content-diff-panel">
              <div class="content-diff-header">
                <NText strong style="font-family: monospace; font-size: 13px">{{ selectedFile }}</NText>
                <NSpace size="small" align="center">
                  <span v-if="addedLines > 0" class="diff-badge diff-added">+{{ addedLines }}</span>
                  <span v-if="removedLines > 0" class="diff-badge diff-removed">-{{ removedLines }}</span>
                  <NButton quaternary size="tiny" @click="selectedFile = null; diffLines = []">
                    {{ t('skills.diff.close') }}
                  </NButton>
                </NSpace>
              </div>

              <NSpin :show="loadingFileContent">
                <div class="diff-viewer">
                  <table v-if="diffLines.length > 0" class="diff-table">
                    <tbody>
                      <tr v-for="(line, idx) in diffLines" :key="idx" :class="'diff-line-' + line.type">
                        <td class="line-no">{{ line.oldLineNo ?? '' }}</td>
                        <td class="line-no">{{ line.newLineNo ?? '' }}</td>
                        <td class="line-prefix">{{ line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' ' }}</td>
                        <td class="line-content"><pre>{{ line.content }}</pre></td>
                      </tr>
                    </tbody>
                  </table>
                  <div v-else-if="!loadingFileContent" class="diff-empty">
                    <NText depth="3">{{ t('skills.diff.noContent') }}</NText>
                  </div>
                </div>
              </NSpin>
            </div>
          </div>
        </NSpin>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.diff-dialog {
  width: 820px;
  max-height: 85vh;
  background: var(--n-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.diff-title {
  font-size: 16px;
  font-weight: 600;
}

.diff-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
}

.diff-summary {
  margin-bottom: 12px;
}

.diff-badge {
  display: inline-block;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.diff-added {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.diff-removed {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.diff-modified {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.diff-unchanged {
  background: rgba(148, 163, 184, 0.15);
  color: #94a3b8;
}

.content-diff-panel {
  margin-top: 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
}

.content-diff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--n-color-modal);
  border-bottom: 1px solid var(--n-border-color);
}

.diff-viewer {
  max-height: 280px;
  overflow-y: auto;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  line-height: 1.6;
}

.diff-table {
  width: 100%;
  border-collapse: collapse;
}

.diff-table td {
  padding: 0 8px;
  vertical-align: top;
}

.line-no {
  width: 40px;
  text-align: right;
  color: var(--n-text-color-3);
  user-select: none;
  opacity: 0.6;
  padding-right: 8px;
  border-right: 1px solid var(--n-border-color);
}

.line-prefix {
  width: 16px;
  user-select: none;
  font-weight: 600;
}

.line-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: inherit;
}

/* Line backgrounds */
.diff-line-added {
  background: rgba(16, 185, 129, 0.12);
}

.diff-line-added .line-prefix {
  color: #10b981;
}

.diff-line-removed {
  background: rgba(239, 68, 68, 0.12);
}

.diff-line-removed .line-prefix {
  color: #ef4444;
}

.diff-line-unchanged {
  /* No background for unchanged lines */
}

.diff-line-unchanged .line-prefix {
  color: var(--n-text-color-3);
}

.diff-empty {
  padding: 24px;
  text-align: center;
}
</style>
