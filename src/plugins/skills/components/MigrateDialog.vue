<script setup lang="ts">
import { NModal, NButton, NSelect, NSpace, NText, NCheckbox, NRadioGroup, NRadio, NSpin, NDivider } from 'naive-ui'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSkillsStore } from '../store'
import { useSettingsStore } from '@/plugins/settings/store'
import type { MigrateSkillItem, ConflictResolution } from '../types'
import SkillDiffViewer from './SkillDiffViewer.vue'

const { t } = useI18n()
const store = useSkillsStore()
const settingsStore = useSettingsStore()

const currentStep = ref(1)
const sourceAgentId = ref<string | null>(null)
const scanning = ref(false)
const selectedNames = ref<Set<string>>(new Set())
const resolutions = ref<Record<string, ConflictResolution>>({})
const migrating = ref(false)
const resultMessage = ref<string | null>(null)
const resultSuccess = ref(true)

// Diff viewer state
const showDiff = ref(false)
const diffSourcePath = ref('')
const diffTargetPath = ref('')

// Available source agents: all enabled agents except current
const sourceAgentOptions = computed(() => {
  const currentId = store.currentAgentId
  return settingsStore.agents
    .filter((a) => a.enabled && a.id !== currentId)
    .map((a) => ({ label: a.name, value: a.id }))
})

// Computed lists
const newSkills = computed(() =>
  store.migrateSkills.filter((s) => s.status === 'newTarget')
)

const sameSkills = computed(() =>
  store.migrateSkills.filter((s) => s.status === 'sameContent')
)

const conflictSkills = computed(() =>
  store.migrateSkills.filter(
    (s) => s.status === 'differentVersion' || s.status === 'contentDiffers'
  )
)

const selectedCount = computed(() => selectedNames.value.size)

const migrateCount = computed(() => {
  let count = 0
  for (const name of selectedNames.value) {
    if (resolutions.value[name] === 'Overwrite') count++
    else {
      const item = store.migrateSkills.find((s) => s.name === name)
      if (item?.status === 'newTarget') count++
    }
  }
  return count
})

const skipCount = computed(() => selectedCount.value - migrateCount.value)

function getStatusInfo(status: MigrateSkillItem['status']) {
  const map: Record<string, { color: string; label: string }> = {
    newTarget: { color: '#10b981', label: t('skills.migrate.statusNew') },
    sameContent: { color: '#94a3b8', label: t('skills.migrate.statusSame') },
    differentVersion: { color: '#f59e0b', label: t('skills.migrate.statusVersion') },
    contentDiffers: { color: '#f97316', label: t('skills.migrate.statusContent') },
  }
  return map[status] ?? { color: '#94a3b8', label: status }
}

async function handleAgentSelect(value: string) {
  sourceAgentId.value = value
  scanning.value = true
  selectedNames.value.clear()
  resolutions.value = {}
  resultMessage.value = null

  try {
    await store.scanAgentSkills(value)
    // Auto-advance if skills found
    if (store.migrateSkills.length > 0) {
      preSelectSkills()
      currentStep.value = 2
    }
  } finally {
    scanning.value = false
  }
}

function preSelectSkills() {
  for (const item of store.migrateSkills) {
    if (item.status === 'newTarget' || item.status === 'differentVersion' || item.status === 'contentDiffers') {
      selectedNames.value.add(item.name)
    }
    if (item.status === 'differentVersion' || item.status === 'contentDiffers') {
      resolutions.value[item.name] = 'Overwrite'
    }
  }
}

function toggleSelectAll() {
  if (selectedNames.value.size === store.migrateSkills.length) {
    selectedNames.value.clear()
  } else {
    selectedNames.value = new Set(store.migrateSkills.map((s) => s.name))
  }
}

function toggleSkill(name: string) {
  const next = new Set(selectedNames.value)
  if (next.has(name)) {
    next.delete(name)
  } else {
    next.add(name)
  }
  selectedNames.value = next
}

function goToStep(step: number) {
  currentStep.value = step
}

function handleViewDiff(sourcePath: string, targetPath: string) {
  diffSourcePath.value = sourcePath
  diffTargetPath.value = targetPath
  showDiff.value = true
}

async function handleConfirm() {
  migrating.value = true
  resultMessage.value = null

  try {
    const result = await store.executeMigration(
      Array.from(selectedNames.value),
      resolutions.value
    )

    if (result.failed.length > 0) {
      resultSuccess.value = false
      resultMessage.value = t('skills.migrate.success', {
        migrated: result.migrated.length,
        failed: result.failed.length,
      })
    } else {
      resultSuccess.value = true
      resultMessage.value = t('skills.migrate.success', {
        migrated: result.migrated.length,
        failed: result.failed.length,
      })
    }
    currentStep.value = 3
  } catch (e) {
    resultSuccess.value = false
    resultMessage.value = t('skills.migrate.failed', {
      error: e instanceof Error ? e.message : String(e),
    })
    currentStep.value = 3
  } finally {
    migrating.value = false
  }
}

function handleClose() {
  currentStep.value = 1
  sourceAgentId.value = null
  selectedNames.value.clear()
  resolutions.value = {}
  resultMessage.value = null
  store.clearMigration()
  store.showMigrateDialog = false
}

// Reset state when dialog opens
watch(() => store.showMigrateDialog, (open) => {
  if (open) {
    currentStep.value = 1
    sourceAgentId.value = null
    selectedNames.value.clear()
    resolutions.value = {}
    resultMessage.value = null
  }
})
</script>

<template>
  <NModal data-testid="migrate-dialog" :show="store.showMigrateDialog" @update:show="(v) => !v && handleClose()">
    <div class="migrate-dialog">
      <!-- Header -->
      <div class="migrate-header">
        <span class="migrate-title">{{ t('skills.migrate.title') }}</span>

        <!-- Step indicator -->
        <div class="step-indicator">
          <div
            v-for="step in 3"
            :key="step"
            class="step-item"
            :class="{ active: currentStep >= step, current: currentStep === step }"
          >
            <span class="step-circle">{{ step }}</span>
            <span v-if="step < 3" class="step-line" />
          </div>
        </div>

        <NButton quaternary size="small" @click="handleClose">
          {{ t('skills.diff.close') }}
        </NButton>
      </div>

      <!-- Step content -->
      <div class="migrate-body">
        <!-- Step 1: Select Source Agent -->
        <div v-if="currentStep === 1" class="step-content">
          <NText class="step-label">{{ t('skills.migrate.step1') }}</NText>

          <NSelect
            :value="sourceAgentId"
            :options="sourceAgentOptions"
            :placeholder="t('skills.migrate.sourceAgentPlaceholder')"
            to="body"
            @update:value="handleAgentSelect"
            style="margin-top: 12px"
          />

          <NSpin v-if="scanning" :show="true" style="margin-top: 16px" />

          <div v-if="!scanning && sourceAgentId && store.migrateSkills.length === 0" class="empty-hint">
            <NText depth="3">{{ t('skills.migrate.noSkills') }}</NText>
          </div>

          <div v-if="sourceAgentOptions.length === 0" class="empty-hint">
            <NText depth="3">{{ t('skills.migrate.noAgents') }}</NText>
          </div>
        </div>

        <!-- Step 2: Select Skills -->
        <div v-if="currentStep === 2" class="step-content">
          <div class="step-header-row">
            <NText class="step-label">{{ t('skills.migrate.step2') }}</NText>
            <NSpace size="small" align="center">
              <NButton text size="small" @click="toggleSelectAll">
                {{ t('skills.migrate.selectAll') }}
              </NButton>
              <NText depth="3" style="font-size: 12px">
                {{ t('skills.migrate.selected', { count: selectedCount }) }}
              </NText>
            </NSpace>
          </div>

          <div class="skills-list">
            <!-- New skills -->
            <div v-if="newSkills.length > 0" class="skill-group">
              <NText depth="3" style="font-size: 12px; font-weight: 600">
                {{ t('skills.migrate.newGroup') }} ({{ newSkills.length }})
              </NText>
              <div
                v-for="skill in newSkills"
                :key="skill.name"
                class="skill-row"
                @click="toggleSkill(skill.name)"
              >
                <NCheckbox
                  :checked="selectedNames.has(skill.name)"
                  @update:checked="() => toggleSkill(skill.name)"
                />
                <span class="skill-name">{{ skill.name }}</span>
                <span v-if="skill.version" class="skill-version">{{ skill.version }}</span>
                <span class="status-badge" :style="{ color: '#10b981', background: 'rgba(16,185,129,0.12)' }">
                  {{ getStatusInfo(skill.status).label }}
                </span>
              </div>
            </div>

            <!-- Conflict skills -->
            <div v-if="conflictSkills.length > 0" class="skill-group">
              <NText depth="3" style="font-size: 12px; font-weight: 600">
                {{ t('skills.migrate.conflictGroup') }} ({{ conflictSkills.length }})
              </NText>
              <div
                v-for="skill in conflictSkills"
                :key="skill.name"
                class="skill-row"
              >
                <NCheckbox
                  :checked="selectedNames.has(skill.name)"
                  @update:checked="() => toggleSkill(skill.name)"
                />
                <span class="skill-name">{{ skill.name }}</span>
                <span v-if="skill.version" class="skill-version">{{ skill.version }}</span>
                <span class="status-badge" :style="{ color: getStatusInfo(skill.status).color, background: getStatusInfo(skill.status).color + '1a' }">
                  {{ getStatusInfo(skill.status).label }}
                </span>
                <NButton
                  text
                  size="tiny"
                  type="info"
                  @click.stop="handleViewDiff(skill.sourcePath, skill.targetPath)"
                >
                  {{ t('skills.migrate.diff') }}
                </NButton>
              </div>
            </div>

            <!-- Same content skills (dimmed) -->
            <div v-if="sameSkills.length > 0" class="skill-group">
              <NText depth="3" style="font-size: 12px; font-weight: 600">
                {{ t('skills.migrate.statusSame') }} ({{ sameSkills.length }})
              </NText>
              <div
                v-for="skill in sameSkills"
                :key="skill.name"
                class="skill-row dimmed"
                @click="toggleSkill(skill.name)"
              >
                <NCheckbox
                  :checked="selectedNames.has(skill.name)"
                  @update:checked="() => toggleSkill(skill.name)"
                />
                <span class="skill-name">{{ skill.name }}</span>
                <span v-if="skill.version" class="skill-version">{{ skill.version }}</span>
                <span class="status-badge" :style="{ color: '#94a3b8', background: 'rgba(148,163,184,0.12)' }">
                  {{ getStatusInfo(skill.status).label }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 3: Confirm Migration -->
        <div v-if="currentStep === 3" class="step-content">
          <NText class="step-label">{{ t('skills.migrate.step3') }}</NText>

          <!-- Result message -->
          <div v-if="resultMessage" class="result-message" :class="{ success: resultSuccess, error: !resultSuccess }">
            <NText :type="resultSuccess ? 'success' : 'error'">{{ resultMessage }}</NText>
          </div>

          <!-- Conflict resolution (before migration) -->
          <template v-if="!resultMessage">
            <!-- New skills: no action needed -->
            <div v-if="newSkills.filter((s) => selectedNames.has(s.name)).length > 0" class="confirm-group">
              <NText depth="3" style="font-size: 12px; font-weight: 600">
                {{ t('skills.migrate.newGroup') }}
              </NText>
              <div v-for="skill in newSkills.filter((s) => selectedNames.has(s.name))" :key="skill.name" class="confirm-row">
                <span class="skill-name">{{ skill.name }}</span>
                <span class="status-badge" :style="{ color: '#10b981', background: 'rgba(16,185,129,0.12)' }">
                  {{ getStatusInfo('newTarget').label }}
                </span>
              </div>
            </div>

            <!-- Conflict skills: resolution -->
            <div v-if="conflictSkills.filter((s) => selectedNames.has(s.name)).length > 0" class="confirm-group">
              <NText depth="3" style="font-size: 12px; font-weight: 600">
                {{ t('skills.migrate.conflictGroup') }}
              </NText>
              <div v-for="skill in conflictSkills.filter((s) => selectedNames.has(s.name))" :key="skill.name" class="confirm-row">
                <span class="skill-name">{{ skill.name }}</span>
                <span class="status-badge" :style="{ color: getStatusInfo(skill.status).color, background: getStatusInfo(skill.status).color + '1a' }">
                  {{ getStatusInfo(skill.status).label }}
                </span>
                <NRadioGroup
                  :value="resolutions[skill.name] ?? 'Skip'"
                  size="small"
                  @update:value="(v: string) => resolutions[skill.name] = v as ConflictResolution"
                >
                  <NSpace size="small">
                    <NRadio value="Skip">{{ t('skills.migrate.skip') }}</NRadio>
                    <NRadio value="Overwrite">{{ t('skills.migrate.overwrite') }}</NRadio>
                  </NSpace>
                </NRadioGroup>
                <NButton
                  text
                  size="tiny"
                  type="info"
                  @click="handleViewDiff(skill.sourcePath, skill.targetPath)"
                >
                  {{ t('skills.migrate.diff') }}
                </NButton>
              </div>
            </div>

            <!-- Summary -->
            <NDivider />
            <div class="summary-bar">
              <NText>{{ t('skills.migrate.summary', { migrate: migrateCount, skip: skipCount }) }}</NText>
            </div>
          </template>
        </div>
      </div>

      <!-- Footer -->
      <div class="migrate-footer">
        <NSpace>
          <NButton v-if="currentStep > 1 && !resultMessage" @click="goToStep(currentStep - 1)">
            {{ t('skills.migrate.prev') }}
          </NButton>
          <NButton
            v-if="currentStep === 2"
            type="primary"
            :disabled="selectedCount === 0"
            @click="goToStep(3)"
          >
            {{ t('skills.migrate.next') }}
          </NButton>
          <NButton
            v-if="currentStep === 3 && !resultMessage"
            type="primary"
            :loading="migrating"
            @click="handleConfirm"
          >
            {{ migrating ? t('skills.migrate.migrating') : t('skills.migrate.confirm') }}
          </NButton>
          <NButton v-if="resultMessage" type="primary" @click="handleClose">
            {{ t('skills.diff.close') }}
          </NButton>
        </NSpace>
      </div>
    </div>
  </NModal>

  <!-- Diff viewer for migration conflicts -->
  <SkillDiffViewer
    :show="showDiff"
    :local-path="diffTargetPath"
    :remote-path="diffSourcePath"
    @close="showDiff = false"
  />
</template>

<style scoped>
.migrate-dialog {
  width: 640px;
  max-height: 80vh;
  background: var(--n-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.migrate-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.migrate-title {
  font-size: 16px;
  font-weight: 600;
}

.step-indicator {
  display: flex;
  align-items: center;
  gap: 0;
}

.step-item {
  display: flex;
  align-items: center;
}

.step-circle {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  background: var(--n-color-modal);
  border: 2px solid var(--n-border-color);
  color: var(--n-text-color-3);
  transition: all 0.2s;
}

.step-item.active .step-circle {
  border-color: #18a058;
  color: #18a058;
}

.step-item.current .step-circle {
  background: #18a058;
  border-color: #18a058;
  color: #fff;
}

.step-line {
  width: 32px;
  height: 2px;
  background: var(--n-border-color);
  margin: 0 4px;
}

.step-item.active .step-line {
  background: #18a058;
}

.migrate-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
}

.step-content {
  display: flex;
  flex-direction: column;
}

.step-label {
  font-size: 14px;
  font-weight: 600;
}

.step-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.skills-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skill-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skill-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.skill-row:hover {
  background: var(--n-color-hover);
}

.skill-row.dimmed {
  opacity: 0.55;
}

.skill-name {
  font-size: 13px;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
}

.skill-version {
  font-size: 11px;
  color: var(--n-text-color-3);
  background: var(--n-color-modal);
  padding: 1px 6px;
  border-radius: 3px;
}

.status-badge {
  display: inline-block;
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 10px;
  font-weight: 500;
  margin-left: auto;
  white-space: nowrap;
}

.empty-hint {
  text-align: center;
  padding: 24px 0;
}

.confirm-group {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.confirm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  background: var(--n-color-modal);
}

.summary-bar {
  text-align: center;
  padding: 8px;
  background: var(--n-color-modal);
  border-radius: 4px;
}

.result-message {
  text-align: center;
  padding: 16px;
  border-radius: 4px;
  margin-bottom: 12px;
}

.result-message.success {
  background: rgba(16, 185, 129, 0.1);
}

.result-message.error {
  background: rgba(239, 68, 68, 0.1);
}

.migrate-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--n-border-color);
  flex-shrink: 0;
}
</style>
