<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NButton,
  NTag,
  NText,
  NEmpty,
  NSpace,
  NPopconfirm,
  NScrollbar,
} from 'naive-ui'
import type { SkillbaseResolution, DependencyStatus } from '../types'

const { t } = useI18n()

const props = defineProps<{
  resolution: SkillbaseResolution
  syncing: boolean
}>()

const emit = defineEmits<{
  sync: []
  regenerate: []
}>()

const hasUnsatisfied = computed(() => {
  return (
    props.resolution.missingCount +
      props.resolution.mismatchCount +
      props.resolution.outdatedCount >
    0
  )
})

function statusType(status: DependencyStatus): 'success' | 'warning' | 'error' | 'info' {
  switch (status) {
    case 'satisfied':
      return 'success'
    case 'missing':
      return 'error'
    case 'versionMismatch':
      return 'warning'
    case 'outdated':
      return 'info'
  }
}
</script>

<template>
  <div class="skillbase-panel">
    <div class="panel-header">
      <div class="panel-title">
        <NText strong>skillbase.json</NText>
        <NText depth="3" style="font-size: 12px; margin-left: 8px">
          {{ resolution.manifest.name }}
        </NText>
      </div>
      <div class="panel-stats">
        <NTag size="small" round :bordered="false" type="success">
          {{ resolution.satisfiedCount }} {{ t('skills.skillbase.satisfied') }}
        </NTag>
        <NTag
          v-if="resolution.missingCount > 0"
          size="small"
          round
          :bordered="false"
          type="error"
        >
          {{ resolution.missingCount }} {{ t('skills.skillbase.missing') }}
        </NTag>
        <NTag
          v-if="resolution.mismatchCount > 0"
          size="small"
          round
          :bordered="false"
          type="warning"
        >
          {{ resolution.mismatchCount }} {{ t('skills.skillbase.mismatch') }}
        </NTag>
        <NTag
          v-if="resolution.outdatedCount > 0"
          size="small"
          round
          :bordered="false"
          type="info"
        >
          {{ resolution.outdatedCount }} {{ t('skills.skillbase.outdated') }}
        </NTag>
      </div>
      <NSpace size="small" align="center">
        <NPopconfirm @positive-click="emit('regenerate')">
          <template #trigger>
            <NButton size="tiny" quaternary>
              {{ t('skills.skillbase.regenerate') }}
            </NButton>
          </template>
          {{ t('skills.skillbase.regenerateConfirm') }}
        </NPopconfirm>
        <NButton
          v-if="hasUnsatisfied"
          size="tiny"
          type="primary"
          :loading="syncing"
          @click="emit('sync')"
        >
          {{ t('skills.skillbase.syncDeps') }}
        </NButton>
      </NSpace>
    </div>
    <NScrollbar style="max-height: 300px">
      <div class="dep-list">
        <div
          v-for="dep in resolution.dependencies"
          :key="dep.reference"
          class="dep-item"
        >
          <span
            class="dep-dot"
            :class="{
              satisfied: dep.status === 'satisfied',
              missing: dep.status === 'missing',
              mismatch: dep.status === 'versionMismatch',
              outdated: dep.status === 'outdated',
            }"
          />
          <span class="dep-ref">{{ dep.reference }}</span>
          <NText depth="3" style="font-size: 12px">{{ dep.versionRange }}</NText>
          <NTag size="tiny" round :bordered="false" :type="statusType(dep.status)">
            {{ t(`skills.skillbase.${dep.status}`) }}
          </NTag>
          <NText
            v-if="dep.installedVersion"
            depth="3"
            style="font-size: 11px; margin-left: auto"
          >
            v{{ dep.installedVersion }}
          </NText>
        </div>
      </div>
    </NScrollbar>
    <NEmpty
      v-if="resolution.dependencies.length === 0"
      :description="t('skills.skillbase.noDependencies')"
      style="padding: 16px 0"
    />
  </div>
</template>

<style scoped>
.skillbase-panel {
  padding: 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 8px;
  background: var(--n-color);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.panel-title {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.panel-stats {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.dep-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dep-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 13px;
}

.dep-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.dep-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dep-dot.satisfied {
  background: #18a058;
}
.dep-dot.missing {
  background: #d03050;
}
.dep-dot.mismatch {
  background: #f0a020;
}
.dep-dot.outdated {
  background: #2080f0;
}

.dep-ref {
  font-family: var(--n-font-family-mono);
  font-size: 12px;
}
</style>
