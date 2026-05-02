<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  NModal,
  NCard,
  NSpin,
  NTree,
  NEmpty,
  NText,
  NDescriptions,
  NDescriptionsItem,
  NTag,
  type TreeOption,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import json from 'highlight.js/lib/languages/json'
import yaml from 'highlight.js/lib/languages/yaml'
import bash from 'highlight.js/lib/languages/bash'
import xml from 'highlight.js/lib/languages/xml'
import css from 'highlight.js/lib/languages/css'
import markdown from 'highlight.js/lib/languages/markdown'
import sql from 'highlight.js/lib/languages/sql'
import type { FileEntry } from '../types'

import 'highlight.js/styles/github.css'

// Register highlight.js languages
hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('json', json)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('html', xml)
hljs.registerLanguage('css', css)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('sql', sql)

// Configure marked to use highlight.js
marked.use({
  renderer: {
    code({ text, lang }: { text: string; lang?: string }) {
      const language = lang && hljs.getLanguage(lang) ? lang : ''
      const highlighted = language
        ? hljs.highlight(text, { language }).value
        : hljs.highlightAuto(text).value
      return `<pre><code class="hljs language-${language}">${highlighted}</code></pre>`
    },
  },
})

const { t } = useI18n()

interface FrontmatterData {
  name: string
  version: string
  description: string
  tags: string[]
  license: string
  updated_at: string
  author: string
  language: string
  repository: string
  trigger: {
    description: string
    tags: string[]
    file_patterns: string[]
    priority?: number
  } | null
  security: {
    permissions: string[]
  } | null
  compatibility: {
    min_context_tokens?: number
    requires: string[]
    models: string[]
  } | null
  dependencies: Record<string, string> | null
  extra: Record<string, string>
}

const props = defineProps<{
  show: boolean
  skillPath: string
  skillName: string
}>()

const emit = defineEmits<{ close: [] }>()

const fileTree = ref<FileEntry[]>([])
const currentFile = ref('SKILL.md')
const rawContent = ref('')
const loading = ref(false)
const treeLoading = ref(false)
const frontmatter = ref<FrontmatterData | null>(null)

const isMarkdown = computed(() => {
  const ext = currentFile.value.split('.').pop()?.toLowerCase()
  return ext === 'md' || ext === 'mdx'
})

function detectLanguage(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    mjs: 'javascript',
    py: 'python',
    json: 'json',
    yaml: 'yaml',
    yml: 'yaml',
    sh: 'bash',
    bash: 'bash',
    xml: 'xml',
    html: 'xml',
    htm: 'xml',
    css: 'css',
    md: 'markdown',
    mdx: 'markdown',
    sql: 'sql',
  }
  return map[ext] || ''
}

function parseFrontmatter(content: string): { fm: FrontmatterData | null; body: string } {
  const trimmed = content.trimStart()
  if (!trimmed.startsWith('---')) return { fm: null, body: content }

  const rest = trimmed.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return { fm: null, body: content }

  const yaml = rest.slice(0, end)
  const body = rest.slice(end + 4).trimStart()

  const fm: FrontmatterData = {
    name: '',
    version: '',
    description: '',
    tags: [],
    license: '',
    updated_at: '',
    author: '',
    language: '',
    repository: '',
    trigger: null,
    security: null,
    compatibility: null,
    dependencies: null,
    extra: {},
  }

  let currentSection = ''
  let currentObj: Record<string, any> | null = null

  for (const line of yaml.split('\n')) {
    const trimmedLine = line.trim()

    if (!trimmedLine || trimmedLine.startsWith('#')) continue

    // Detect nested section headers (indented objects)
    if (!line.startsWith(' ') && !line.startsWith('\t')) {
      currentSection = ''
      currentObj = null
    }

    const colon = trimmedLine.indexOf(':')
    if (colon === -1) continue
    const key = trimmedLine.slice(0, colon).trim()
    let val = trimmedLine.slice(colon + 1).trim()

    // Top-level keys
    if (!line.startsWith(' ') && !line.startsWith('\t')) {
      switch (key) {
        case 'name':
          fm.name = val
          break
        case 'version':
          fm.version = val
          break
        case 'description':
          fm.description = val
          break
        case 'license':
          fm.license = val
          break
        case 'updated_at':
          fm.updated_at = val
          break
        case 'author':
          fm.author = val
          break
        case 'language':
          fm.language = val
          break
        case 'repository':
          fm.repository = val
          break
        case 'tags':
          fm.tags = parseYamlArray(val)
          break
        case 'trigger':
          currentSection = 'trigger'
          currentObj = { description: '', tags: [], file_patterns: [] }
          fm.trigger = currentObj as any
          break
        case 'security':
          currentSection = 'security'
          currentObj = { permissions: [] }
          fm.security = currentObj as any
          break
        case 'compatibility':
          currentSection = 'compatibility'
          currentObj = { requires: [], models: [] }
          fm.compatibility = currentObj as any
          break
        case 'dependencies':
          currentSection = 'dependencies'
          fm.dependencies = {}
          break
        default:
          fm.extra[key] = val
      }
      continue
    }

    // Nested keys (indented under trigger/security/compatibility/dependencies)
    if (currentSection === 'trigger' && currentObj) {
      if (key === 'description') currentObj.description = val
      else if (key === 'tags') currentObj.tags = parseYamlArray(val)
      else if (key === 'file_patterns') currentObj.file_patterns = parseYamlArray(val)
      else if (key === 'priority') currentObj.priority = parseInt(val, 10) || undefined
    } else if (currentSection === 'security' && currentObj) {
      if (key === 'permissions') currentObj.permissions = parseYamlArray(val)
    } else if (currentSection === 'compatibility' && currentObj) {
      if (key === 'min_context_tokens') currentObj.min_context_tokens = parseInt(val, 10) || undefined
      else if (key === 'requires') currentObj.requires = parseYamlArray(val)
      else if (key === 'models') currentObj.models = parseYamlArray(val)
    } else if (currentSection === 'dependencies' && fm.dependencies) {
      fm.dependencies[key] = val
    }
  }

  return { fm, body }
}

function parseYamlArray(val: string): string[] {
  if (val.startsWith('[') && val.endsWith(']')) {
    return val
      .slice(1, -1)
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
  }
  return []
}

function stripFrontmatter(content: string): string {
  const trimmed = content.trimStart()
  if (!trimmed.startsWith('---')) return content
  const rest = trimmed.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return content
  return rest.slice(end + 4).trimStart()
}

const renderedContent = computed(() => {
  if (!isMarkdown.value) return ''
  try {
    const cleaned = stripFrontmatter(rawContent.value)
    return marked(cleaned) as string
  } catch {
    return `<p>${t('skills.preview.renderFailed')}</p>`
  }
})

const highlightedCode = computed(() => {
  if (isMarkdown.value || !rawContent.value) return ''
  const lang = detectLanguage(currentFile.value)
  try {
    if (lang && hljs.getLanguage(lang)) {
      return hljs.highlight(rawContent.value, { language: lang }).value
    }
    return hljs.highlightAuto(rawContent.value).value
  } catch {
    return rawContent.value
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
  }
})

function toTreeOption(entry: FileEntry): TreeOption {
  const icon = entry.isDir ? '📁 ' : getFileIcon(entry.name)
  return {
    key: entry.path,
    label: icon + entry.name,
    isLeaf: !entry.isDir,
    children: entry.children?.map(toTreeOption),
  }
}

function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase()
  const icons: Record<string, string> = {
    md: '📄',
    py: '🐍',
    cs: '💻',
    ts: '📜',
    js: '📜',
    json: '📋',
    yaml: '📋',
    yml: '📋',
    txt: '📝',
  }
  return icons[ext || ''] || '📄'
}

async function loadFileTree() {
  if (!props.skillPath) return

  treeLoading.value = true
  try {
    fileTree.value = await invoke<FileEntry[]>('get_skill_file_tree', {
      skillPath: props.skillPath,
    })
  } catch (e) {
    console.error('[SkillPreviewModal] Failed to load file tree:', e)
    fileTree.value = []
  } finally {
    treeLoading.value = false
  }
}

async function loadFile(path: string) {
  loading.value = true
  try {
    rawContent.value = await invoke<string>('read_skill_file', {
      filePath: path,
    })
    currentFile.value = path
    const fileName = path.split(/[/\\]/).pop() || ''
    if (fileName === 'SKILL.md') {
      const { fm } = parseFrontmatter(rawContent.value)
      frontmatter.value = fm
    } else {
      frontmatter.value = null
    }
  } catch (e: any) {
    rawContent.value = t('skills.preview.loadFailed', { error: e })
    frontmatter.value = null
  } finally {
    loading.value = false
  }
}

function handleFileSelect(keys: string[]) {
  if (keys.length > 0) {
    const path = keys[0]
    // Only load files, not directories
    const entry = findEntryByPath(fileTree.value, path)
    if (entry && !entry.isDir) {
      loadFile(path)
    }
  }
}

function findEntryByPath(entries: FileEntry[], path: string): FileEntry | null {
  for (const entry of entries) {
    if (entry.path === path) return entry
    if (entry.isDir && entry.children) {
      const found = findEntryByPath(entry.children, path)
      if (found) return found
    }
  }
  return null
}

const treeOptions = computed(() => fileTree.value.map(toTreeOption))
const defaultExpandedKeys = computed(() => [] as string[])

// Find SKILL.md in tree recursively
function findSkillMd(entries: FileEntry[]): string | null {
  for (const entry of entries) {
    if (entry.name === 'SKILL.md' && !entry.isDir) {
      return entry.path
    }
    if (entry.isDir && entry.children) {
      const found = findSkillMd(entry.children)
      if (found) return found
    }
  }
  return null
}

// Watch for modal open
watch(
  () => props.show,
  async (show) => {
    if (show && props.skillPath) {
      // Load file tree
      await loadFileTree()

      // Auto-select SKILL.md if exists
      const skillMdPath = findSkillMd(fileTree.value)
      if (skillMdPath) {
        await loadFile(skillMdPath)
      }
    }
  }
)
</script>

<template>
  <n-modal
    :show="props.show"
    :mask-closable="true"
    @update:show="(v) => !v && emit('close')"
  >
    <n-card
      style="width: 800px; max-height: 80vh"
      :title="props.skillName"
      size="small"
      closable
      @close="emit('close')"
    >
      <div class="preview-layout">
        <!-- Left panel: file tree -->
        <div class="file-tree-panel">
          <n-text depth="3" style="font-size: 12px; padding: 8px; display: block">
            {{ t('skills.preview.fileList') }}
          </n-text>
          <n-spin :show="treeLoading" size="small">
            <n-tree
              v-if="treeOptions.length > 0"
              :data="treeOptions"
              :default-expanded-keys="defaultExpandedKeys"
              :selected-keys="[currentFile]"
              block-line
              selectable
              @update:selected-keys="handleFileSelect"
              style="font-size: 13px"
              key-field="key"
              children-field="children"
            />
            <n-empty v-else :description="t('skills.preview.noFiles')" size="small" />
          </n-spin>
        </div>

        <!-- Right panel: content viewer -->
        <div class="file-content-panel">
          <div class="file-tab">
            <n-text strong style="font-size: 13px">{{ currentFile }}</n-text>
          </div>
          <div class="file-content-scroll">
            <n-spin :show="loading" size="small">
              <!-- Frontmatter card for SKILL.md -->
              <div v-if="frontmatter" class="fm-card">
                <n-descriptions label-placement="left" bordered size="small" :column="1">
                  <n-descriptions-item :label="t('skills.preview.name')">
                    <n-text strong>{{ frontmatter.name }}</n-text>
                  </n-descriptions-item>
                  <n-descriptions-item :label="t('skills.preview.version')">
                    <n-text code>{{ frontmatter.version }}</n-text>
                  </n-descriptions-item>
                  <n-descriptions-item :label="t('skills.preview.description')">
                    {{ frontmatter.description }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.tags.length > 0" :label="t('skills.preview.tags')">
                    <n-tag
                      v-for="tag in frontmatter.tags"
                      :key="tag"
                      size="small"
                      round
                      type="info"
                      style="margin-right: 4px"
                    >
                      {{ tag }}
                    </n-tag>
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.license" :label="t('skills.preview.license')">
                    {{ frontmatter.license }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.updated_at" :label="t('skills.preview.updatedAt')">
                    {{ frontmatter.updated_at }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.author" :label="t('skills.preview.author')">
                    {{ frontmatter.author }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.language" :label="t('skills.preview.language')">
                    {{ frontmatter.language }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.repository" :label="t('skills.preview.repository')">
                    {{ frontmatter.repository }}
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.trigger" :label="t('skills.preview.trigger')">
                    <div>
                      <n-text>{{ frontmatter.trigger.description }}</n-text>
                      <div v-if="frontmatter.trigger.tags.length > 0" style="margin-top: 4px">
                        <n-tag
                          v-for="tag in frontmatter.trigger.tags"
                          :key="'t-' + tag"
                          size="small"
                          round
                          type="info"
                          style="margin-right: 4px"
                        >
                          {{ tag }}
                        </n-tag>
                      </div>
                      <div
                        v-if="frontmatter.trigger.file_patterns.length > 0"
                        style="margin-top: 4px; font-size: 12px"
                      >
                        <n-text depth="3">{{ frontmatter.trigger.file_patterns.join(', ') }}</n-text>
                      </div>
                    </div>
                  </n-descriptions-item>
                  <n-descriptions-item
                    v-if="frontmatter.security && frontmatter.security.permissions.length > 0"
                    :label="t('skills.preview.security')"
                  >
                    <n-tag
                      v-for="perm in frontmatter.security.permissions"
                      :key="perm"
                      size="small"
                      round
                      :type="perm.includes('bash') || perm.includes('delete') ? 'warning' : 'default'"
                      style="margin-right: 4px"
                    >
                      {{ perm }}
                    </n-tag>
                  </n-descriptions-item>
                  <n-descriptions-item v-if="frontmatter.compatibility" :label="t('skills.preview.compatibility')">
                    <div style="font-size: 13px">
                      <div v-if="frontmatter.compatibility.min_context_tokens">
                        min_context_tokens: {{ frontmatter.compatibility.min_context_tokens }}
                      </div>
                      <div v-if="frontmatter.compatibility.requires.length > 0">
                        requires: {{ frontmatter.compatibility.requires.join(', ') }}
                      </div>
                      <div v-if="frontmatter.compatibility.models.length > 0">
                        models: {{ frontmatter.compatibility.models.join(', ') }}
                      </div>
                    </div>
                  </n-descriptions-item>
                  <n-descriptions-item
                    v-if="frontmatter.dependencies && Object.keys(frontmatter.dependencies).length > 0"
                    :label="t('skills.preview.dependencies')"
                  >
                    <div style="font-size: 13px">
                      <div v-for="(ver, dep) in frontmatter.dependencies" :key="dep">
                        <n-text code>{{ dep }}</n-text>
                        <n-text depth="3">{{ ver }}</n-text>
                      </div>
                    </div>
                  </n-descriptions-item>
                  <n-descriptions-item v-for="(val, key) in frontmatter.extra" :key="key" :label="key">
                    {{ val }}
                  </n-descriptions-item>
                </n-descriptions>
              </div>

              <!-- Rendered content -->
              <div v-if="isMarkdown && renderedContent" class="md-content" v-html="renderedContent" />
              <pre
                v-else-if="rawContent && !isMarkdown"
                class="raw-content hljs"
                v-html="highlightedCode"
              ></pre>
              <n-empty
                v-else-if="!isMarkdown && !rawContent"
                :description="t('skills.preview.selectFile')"
                size="small"
              />
            </n-spin>
          </div>
        </div>
      </div>
    </n-card>
  </n-modal>
</template>

<style scoped>
.preview-layout {
  display: flex;
  gap: 0;
  height: 500px;
  border: 1px solid var(--n-border-color);
  border-radius: var(--n-border-radius);
  overflow: hidden;
}

.file-tree-panel {
  width: 200px;
  min-width: 200px;
  border-right: 1px solid var(--n-border-color);
  overflow-y: auto;
  background: var(--n-color-modal);
}

.file-content-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.file-tab {
  padding: 6px 12px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color-modal);
  flex-shrink: 0;
}

.file-content-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.fm-card {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color-modal);
}

.md-content {
  padding: 16px;
  font-size: 14px;
  line-height: 1.6;
}

.md-content :deep(h1),
.md-content :deep(h2),
.md-content :deep(h3) {
  margin-top: 16px;
  margin-bottom: 8px;
}

.md-content :deep(code) {
  background: var(--n-code-color);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 13px;
}

.md-content :deep(pre) {
  background: var(--n-code-color);
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
}

.md-content :deep(pre code) {
  background: none;
  padding: 0;
}

.md-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
}

.md-content :deep(th),
.md-content :deep(td) {
  border: 1px solid var(--n-border-color);
  padding: 6px 10px;
  text-align: left;
}

.raw-content {
  padding: 16px;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
  background: var(--n-code-color);
}
</style>
