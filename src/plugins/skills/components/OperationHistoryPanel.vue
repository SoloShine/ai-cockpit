<script setup lang="ts">
import { NDrawer, NDrawerContent, NButton, NSpace, NText, NTag, NEmpty, NSpin, NScrollbar, NPopconfirm, NIcon } from 'naive-ui'
import { watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSkillsStore } from '../store'
import { TimeOutline } from '@vicons/ionicons5'

const { t } = useI18n()
const store = useSkillsStore()

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

function getTypeTag(type: 'install' | 'update' | 'uninstall') {
  const map = {
    install: { type: 'success' as const, label: t('skills.history.install') },
    update: { type: 'warning' as const, label: t('skills.history.update') },
    uninstall: { type: 'error' as const, label: t('skills.history.uninstall') },
  }
  const info = map[type]
  return info
}

function formatTimestamp(ts: string): string {
  try {
    const date = new Date(ts)
    return date.toLocaleString()
  } catch {
    return ts
  }
}

async function handleRollback(id: string) {
  try {
    await store.rollbackOperation(id)
  } catch (e) {
    console.error('[OperationHistoryPanel] Rollback failed:', e)
  }
}

async function handleClear() {
  try {
    await store.clearHistory()
  } catch (e) {
    console.error('[OperationHistoryPanel] Clear failed:', e)
  }
}

watch(() => props.show, (show) => {
  if (show) {
    store.getOperationHistory(100)
  }
})
</script>

<template>
  <NDrawer :show="props.show" :width="420" placement="right" @update:show="(v) => !v && emit('close')">
    <NDrawerContent>
      <template #header>
        <NSpace justify="space-between" align="center" style="width: 100%">
          <NText strong style="font-size: 16px">{{ t('skills.history.title') }}</NText>
          <NPopconfirm @positive-click="handleClear">
            <template #trigger>
              <NButton size="tiny" type="error" ghost>
                {{ t('skills.history.clear') }}
              </NButton>
            </template>
            {{ t('skills.history.clearConfirm') }}
          </NPopconfirm>
        </NSpace>
      </template>

      <NSpin :show="store.operationHistory.length === 0 && props.show" style="min-height: 100px">
        <NScrollbar style="max-height: calc(100vh - 120px)">
          <div v-if="store.operationHistory.length === 0" style="padding: 40px 0; text-align: center">
            <NEmpty :description="t('skills.history.empty')" />
          </div>

          <div v-else class="history-list">
            <div
              v-for="record in store.operationHistory"
              :key="record.id"
              class="history-item"
            >
              <div class="history-item-header">
                <NSpace size="small" align="center">
                  <NTag
                    :type="getTypeTag(record.operationType).type"
                    size="small"
                    :bordered="false"
                  >
                    {{ getTypeTag(record.operationType).label }}
                  </NTag>
                  <NText strong>{{ record.skillName }}</NText>
                </NSpace>

                <NSpace size="small" align="center">
                  <NTag v-if="record.rolledBack" size="small" type="default" :bordered="false">
                    {{ t('skills.history.rolledBack') }}
                  </NTag>
                  <NPopconfirm
                    v-if="record.canRollback && !record.rolledBack"
                    @positive-click="handleRollback(record.id)"
                  >
                    <template #trigger>
                      <NButton size="tiny" type="warning" ghost>
                        {{ t('skills.history.rollback') }}
                      </NButton>
                    </template>
                    {{ t('skills.history.rollbackConfirm') }}
                  </NPopconfirm>
                </NSpace>
              </div>

              <div class="history-item-body">
                <NText depth="3" style="font-family: monospace; font-size: 12px; word-break: break-all">
                  {{ record.targetPath }}
                </NText>
              </div>

              <div class="history-item-footer">
                <NSpace size="small" align="center">
                  <NIcon size="14" :component="TimeOutline" style="opacity: 0.5" />
                  <NText depth="3" style="font-size: 12px">
                    {{ formatTimestamp(record.timestamp) }}
                  </NText>
                </NSpace>
              </div>
            </div>
          </div>
        </NScrollbar>
      </NSpin>
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped>
.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.history-item {
  padding: 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 6px;
  transition: background 0.2s;
}

.history-item:hover {
  background: var(--n-color-hover);
}

.history-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.history-item-body {
  margin-bottom: 6px;
}

.history-item-footer {
  display: flex;
  align-items: center;
}
</style>
