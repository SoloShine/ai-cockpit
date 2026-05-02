<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import {
  NLayout,
  NLayoutSider,
  NLayoutContent,
  NMenu,
  NH1,
  NH2,
  NH4,
  NText,
  NTag,
  NCode,
  NButton,
  NDivider,
  NTable,
  NDescriptions,
  NDescriptionsItem,
  type MenuOption,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { CopyOutline, CheckmarkOutline } from '@vicons/ionicons5'
import hljs from 'highlight.js/lib/core'
import yaml from 'highlight.js/lib/languages/yaml'
import bash from 'highlight.js/lib/languages/bash'
import markdown from 'highlight.js/lib/languages/markdown'
import json from 'highlight.js/lib/languages/json'

import 'highlight.js/styles/github.css'

hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('json', json)

const { t } = useI18n()

// --- Section definitions ---
const sectionIds = [
  'directory',
  'frontmatter',
  'trigger',
  'security',
  'dependencies',
  'body',
  'example',
  'publishing',
] as const

type SectionId = (typeof sectionIds)[number]

const activeSection = ref<SectionId>('directory')
const copiedIndex = ref<number | null>(null)

// --- TOC menu ---
function getTocMenuOptions(): MenuOption[] {
  return sectionIds.map((id) => ({
    key: id,
    label: t(`skills.guide.${id}`),
  }))
}

// --- Scroll tracking ---
const observerMap = new Map<string, IntersectionObserverEntry>()

function setupObserver() {
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        observerMap.set(entry.target.id, entry)
      }
      // Find the topmost visible section
      let topId: SectionId = 'directory'
      let topY = Infinity
      for (const id of sectionIds) {
        const e = observerMap.get(id)
        if (e && e.isIntersecting && e.boundingClientRect.top < topY) {
          topY = e.boundingClientRect.top
          topId = id as SectionId
        }
      }
      activeSection.value = topId
    },
    { rootMargin: '-80px 0px -60% 0px', threshold: 0 }
  )
  for (const id of sectionIds) {
    const el = document.getElementById(id)
    if (el) observer.observe(el)
  }
  return observer
}

let observer: IntersectionObserver | null = null

onMounted(() => {
  nextTick(() => {
    observer = setupObserver()
  })
})

onBeforeUnmount(() => {
  observer?.disconnect()
})

function handleTocSelect(key: string) {
  const el = document.getElementById(key)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

// --- Code copy ---
async function copyCode(text: string, index: number) {
  try {
    await navigator.clipboard.writeText(text)
    copiedIndex.value = index
    setTimeout(() => {
      copiedIndex.value = null
    }, 2000)
  } catch {
    // fallback: ignore
  }
}

function highlight(code: string, lang: string): string {
  try {
    if (hljs.getLanguage(lang)) {
      return hljs.highlight(code, { language: lang }).value
    }
    return hljs.highlightAuto(code).value
  } catch {
    return code.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }
}

// --- Static content data ---

const directoryExample = `my-skill/
├── SKILL.md          # Required: skill definition
├── templates/
│   └── config.yaml   # Optional: template files
├── examples/
│   └── demo.md       # Optional: example files
└── scripts/
    └── setup.sh      # Optional: helper scripts`

const frontmatterFields: { field: string; type: string; required: boolean; desc: string }[] = [
  { field: 'name', type: 'string', required: true, desc: 'Skill name (unique identifier)' },
  { field: 'version', type: 'string', required: true, desc: 'Semver version, e.g. "1.0.0"' },
  { field: 'description', type: 'string', required: true, desc: 'One-line description of the skill' },
  { field: 'tags', type: 'string[]', required: true, desc: 'Tags for categorization and search' },
  { field: 'license', type: 'string', required: false, desc: 'License identifier, e.g. "MIT"' },
  { field: 'updated_at', type: 'string', required: false, desc: 'Last update date (YYYY-MM-DD)' },
  { field: 'author', type: 'string', required: false, desc: 'Author name or handle' },
  { field: 'language', type: 'string', required: false, desc: 'Primary language, e.g. "zh-CN"' },
  { field: 'repository', type: 'string', required: false, desc: 'Git repository URL' },
  { field: 'trigger', type: 'object', required: false, desc: 'Trigger conditions (see Trigger section)' },
  { field: 'security', type: 'object', required: false, desc: 'Security permissions (see Security section)' },
  { field: 'compatibility', type: 'object', required: false, desc: 'Compatibility requirements' },
  { field: 'dependencies', type: 'object', required: false, desc: 'Dependency version map' },
]

const triggerExample = `trigger:
  description: "When user wants to deploy to production"
  tags:
    - deploy
    - release
    - production
  file_patterns:
    - "Dockerfile"
    - "docker-compose*.yml"
    - ".github/workflows/*"
  priority: 10`

const securityExample = `security:
  permissions:
    - bash          # Run shell commands
    - file-read     # Read files
    - file-write    # Write or modify files
    - delete        # Delete files or resources
    - network       # Make network requests
    - env-read      # Read environment variables`

const securityPermissions: { name: string; desc: string; risk: 'low' | 'medium' | 'high' }[] = [
  { name: 'bash', desc: 'Execute shell commands via bash/sh', risk: 'high' },
  { name: 'file-read', desc: 'Read file contents from the filesystem', risk: 'low' },
  { name: 'file-write', desc: 'Create or modify files', risk: 'medium' },
  { name: 'delete', desc: 'Delete files or directories', risk: 'high' },
  { name: 'network', desc: 'Make HTTP/network requests', risk: 'medium' },
  { name: 'env-read', desc: 'Read environment variables', risk: 'medium' },
]

const dependenciesExample = `dependencies:
  node: ">=18.0.0"
  git: ">=2.30"
  my-other-skill: "^1.0.0"`

const skillbaseExample = `{
  "skills": {
    "my-other-skill": {
      "version": "1.2.0",
      "path": "~/.claude/skills/my-other-skill"
    }
  }
}`

const bodyExample = `<instructions>
You are a deployment assistant. Follow these steps when the user wants to deploy:

1. Check the current git branch and status
2. Run the test suite to ensure everything passes
3. Build the production bundle
4. Deploy using the configured method
</instructions>

<examples>
<example>
User: Deploy to production
Assistant: I'll help you deploy to production. Let me check the current state first.
  $ git status
  $ npm test
  $ npm run build
  Deployment complete!
</example>
</examples>

<constraints>
- Never deploy from a dirty working tree
- Always run tests before deploying
- Confirm with the user before executing destructive operations
</constraints>`

const fullExample = `---
name: deploy-helper
version: "1.0.0"
description: "Production deployment assistant with safety checks"
tags:
  - deploy
  - production
  - devops
license: MIT
updated_at: "2025-01-15"
author: devops-team
language: zh-CN
repository: https://github.com/example/deploy-helper

trigger:
  description: "When user wants to deploy applications"
  tags:
    - deploy
    - release
    - ship
  file_patterns:
    - "Dockerfile"
    - "docker-compose*.yml"
    - "*.tf"
  priority: 10

security:
  permissions:
    - bash
    - file-read
    - file-write
    - network

compatibility:
  min_context_tokens: 8000
  requires:
    - git
  models:
    - claude-3.5-sonnet
    - gpt-4

dependencies:
  git: ">=2.30"
---

# Deploy Helper

## Overview

This skill provides guided production deployment with built-in safety checks.

## Instructions

<instructions>
When the user requests a deployment:

1. Verify working tree is clean
2. Confirm target environment
3. Run pre-deploy checks (tests, lint, build)
4. Execute deployment
5. Verify deployment succeeded
</instructions>

## Examples

<examples>
<example>
User: Deploy to production
Assistant: I will deploy to production. Let me verify the prerequisites first.
  $ git status --porcelain
  $ npm test
  All checks passed. Proceeding with deployment...
</example>
</examples>

## Constraints

<constraints>
- Never deploy on Fridays
- Always require user confirmation
- Rollback automatically on failure
</constraints>`
</script>

<template>
  <div class="guide-page">
    <n-layout has-sider position="absolute">
      <!-- TOC Sidebar -->
      <n-layout-sider
        :width="220"
        :native-scrollbar="false"
        bordered
        class="guide-sider"
      >
        <div class="sider-header">
          <n-h4 style="margin: 0; padding: 16px 16px 8px">
            {{ t('skills.guide.toc') }}
          </n-h4>
        </div>
        <n-menu
          :value="activeSection"
          :options="getTocMenuOptions()"
          @update:value="handleTocSelect"
        />
      </n-layout-sider>

      <!-- Content area -->
      <n-layout-content :native-scrollbar="false" class="guide-content">
        <div class="content-inner">
          <n-h1>{{ t('skills.guide.title') }}</n-h1>

          <!-- Section 1: Directory Structure -->
          <section id="directory" class="guide-section">
            <n-h2>{{ t('skills.guide.directory') }}</n-h2>
            <n-text>
              每个 Skill 是一个独立目录，核心文件为 SKILL.md。目录结构如下：
            </n-text>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(directoryExample, 0)"
              >
                <template #icon>
                  <component :is="copiedIndex === 0 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 0 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(directoryExample, 'bash')" language="bash" :word-wrap="true" />
            </div>
            <n-text depth="3">
              SKILL.md 是唯一必需的文件，其余为可选内容。
            </n-text>
          </section>

          <n-divider />

          <!-- Section 2: Frontmatter Fields -->
          <section id="frontmatter" class="guide-section">
            <n-h2>{{ t('skills.guide.frontmatter') }}</n-h2>
            <n-text>
              SKILL.md 文件以 YAML frontmatter 开头，用 <n-text code>---</n-text> 分隔。以下是所有支持的字段：
            </n-text>
            <n-table :bordered="true" :single-line="false" size="small" class="field-table">
              <thead>
                <tr>
                  <th>{{ t('skills.guide.field') }}</th>
                  <th>{{ t('skills.guide.type') }}</th>
                  <th>{{ t('skills.guide.required') }}?</th>
                  <th>{{ t('skills.guide.description') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in frontmatterFields" :key="row.field">
                  <td><n-text code>{{ row.field }}</n-text></td>
                  <td><n-text depth="3">{{ row.type }}</n-text></td>
                  <td>
                    <n-tag :type="row.required ? 'error' : 'default'" size="small" round>
                      {{ row.required ? t('skills.guide.required') : t('skills.guide.optional') }}
                    </n-tag>
                  </td>
                  <td>{{ row.desc }}</td>
                </tr>
              </tbody>
            </n-table>
          </section>

          <n-divider />

          <!-- Section 3: Trigger -->
          <section id="trigger" class="guide-section">
            <n-h2>{{ t('skills.guide.trigger') }}</n-h2>
            <n-text>
              Trigger 定义了 Agent 何时激活此 Skill。通过描述、标签和文件模式匹配来触发。
            </n-text>
            <n-h4>字段说明</n-h4>
            <n-descriptions label-placement="left" bordered size="small" :column="1">
              <n-descriptions-item label="description">
                触发条件的自然语言描述，供 Agent 理解何时使用此 Skill
              </n-descriptions-item>
              <n-descriptions-item label="tags">
                关键词标签数组，用于语义匹配。Agent 会根据上下文中的关键词匹配 Skill
              </n-descriptions-item>
              <n-descriptions-item label="file_patterns">
                文件路径 glob 模式数组。当工作目录中存在匹配的文件时触发
              </n-descriptions-item>
              <n-descriptions-item label="priority">
                优先级数值（选填），数值越小优先级越高。多个 Skill 匹配时，优先级高的优先使用
              </n-descriptions-item>
            </n-descriptions>
            <n-h4>示例</n-h4>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(triggerExample, 1)"
              >
                <template #icon>
                  <component :is="copiedIndex === 1 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 1 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(triggerExample, 'yaml')" language="yaml" :word-wrap="true" />
            </div>
          </section>

          <n-divider />

          <!-- Section 4: Security -->
          <section id="security" class="guide-section">
            <n-h2>{{ t('skills.guide.security') }}</n-h2>
            <n-text>
              明确声明 Skill 需要的权限，帮助用户了解 Skill 的能力边界。所有权限必须显式声明。
            </n-text>
            <n-h4>可用权限</n-h4>
            <n-table :bordered="true" :single-line="false" size="small" class="field-table">
              <thead>
                <tr>
                  <th>权限</th>
                  <th>说明</th>
                  <th>风险等级</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="perm in securityPermissions" :key="perm.name">
                  <td><n-text code>{{ perm.name }}</n-text></td>
                  <td>{{ perm.desc }}</td>
                  <td>
                    <n-tag
                      :type="perm.risk === 'high' ? 'error' : perm.risk === 'medium' ? 'warning' : 'success'"
                      size="small"
                      round
                    >
                      {{ perm.risk === 'high' ? '高' : perm.risk === 'medium' ? '中' : '低' }}
                    </n-tag>
                  </td>
                </tr>
              </tbody>
            </n-table>
            <n-h4>示例</n-h4>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(securityExample, 2)"
              >
                <template #icon>
                  <component :is="copiedIndex === 2 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 2 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(securityExample, 'yaml')" language="yaml" :word-wrap="true" />
            </div>
          </section>

          <n-divider />

          <!-- Section 5: Dependencies -->
          <section id="dependencies" class="guide-section">
            <n-h2>{{ t('skills.guide.dependencies') }}</n-h2>
            <n-text>
              声明 Skill 运行所需的外部依赖，包括工具版本和其他 Skill 依赖。
            </n-text>
            <n-h4>在 Frontmatter 中声明</n-h4>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(dependenciesExample, 3)"
              >
                <template #icon>
                  <component :is="copiedIndex === 3 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 3 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(dependenciesExample, 'yaml')" language="yaml" :word-wrap="true" />
            </div>
            <n-h4>skillbase.json</n-h4>
            <n-text>
              当 Skill 依赖其他 Skill 时，需要在 skillbase.json 中注册依赖关系：
            </n-text>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(skillbaseExample, 4)"
              >
                <template #icon>
                  <component :is="copiedIndex === 4 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 4 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(skillbaseExample, 'json')" language="json" :word-wrap="true" />
            </div>
          </section>

          <n-divider />

          <!-- Section 6: Body Writing Guide -->
          <section id="body" class="guide-section">
            <n-h2>{{ t('skills.guide.body') }}</n-h2>
            <n-text>
              Frontmatter 之后的内容是 Markdown 格式的 Body，是 Skill 的核心指令。使用 XML 标签组织不同类型的内容，使 Agent 能准确解析。
            </n-text>
            <n-h4>XML 标签参考</n-h4>
            <n-descriptions label-placement="left" bordered size="small" :column="1">
              <n-descriptions-item label="<instructions>">
                核心指令区域。定义 Agent 的行为规则和步骤，是 Skill 的主要逻辑
              </n-descriptions-item>
              <n-descriptions-item label="<examples>">
                示例对话区域。提供典型的用户输入和 Agent 输出模式，帮助 Agent 理解预期行为
              </n-descriptions-item>
              <n-descriptions-item label="<constraints>">
                约束条件。列出 Agent 必须遵守的限制和规则，确保安全可控
              </n-descriptions-item>
            </n-descriptions>
            <n-h4>示例</n-h4>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(bodyExample, 5)"
              >
                <template #icon>
                  <component :is="copiedIndex === 5 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 5 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(bodyExample, 'markdown')" language="markdown" :word-wrap="true" />
            </div>
          </section>

          <n-divider />

          <!-- Section 7: Full Example -->
          <section id="example" class="guide-section">
            <n-h2>{{ t('skills.guide.example') }}</n-h2>
            <n-text>
              以下是一个完整的 SKILL.md 文件示例，包含 Frontmatter 和 Body：
            </n-text>
            <div class="code-block-wrapper">
              <n-button
                quaternary
                size="tiny"
                class="copy-btn"
                @click="copyCode(fullExample, 6)"
              >
                <template #icon>
                  <component :is="copiedIndex === 6 ? CheckmarkOutline : CopyOutline" />
                </template>
                {{ copiedIndex === 6 ? t('skills.guide.copied') : t('skills.guide.copyCode') }}
              </n-button>
              <n-code :code="highlight(fullExample, 'yaml')" language="yaml" :word-wrap="true" />
            </div>
          </section>

          <n-divider />

          <!-- Section 8: Validation & Publishing -->
          <section id="publishing" class="guide-section">
            <n-h2>{{ t('skills.guide.publishing') }}</n-h2>
            <n-h4>验证 Skill</n-h4>
            <n-text>
              发布前请确保：
            </n-text>
            <ul class="guide-list">
              <li>Frontmatter 格式正确，必填字段齐全（name, version, description, tags）</li>
              <li>version 遵循 Semver 规范（如 1.0.0）</li>
              <li>security.permissions 声明了所有需要的权限</li>
              <li>Body 中的 XML 标签正确闭合</li>
              <li>trigger 描述清晰，能被 Agent 正确匹配</li>
              <li>如有依赖，dependencies 和 skillbase.json 保持一致</li>
            </ul>

            <n-h4>发布流程</n-h4>
            <ol class="guide-list">
              <li>将 Skill 目录提交到 Git 仓库</li>
              <li>确保仓库地址在 Frontmatter 的 repository 字段中</li>
              <li>在 Skill 管理工具中添加仓库源</li>
              <li>同步仓库后，其他用户即可发现并安装此 Skill</li>
            </ol>

            <n-h4>版本更新</n-h4>
            <n-text>
              更新 Skill 时，请同时更新 <n-text code>version</n-text> 和 <n-text code>updated_at</n-text> 字段。
              工具会通过版本号对比来检测更新。
            </n-text>
          </section>

          <div style="height: 80px" />
        </div>
      </n-layout-content>
    </n-layout>
  </div>
</template>

<style scoped>
.guide-page {
  height: calc(100vh - 120px);
  position: relative;
}

.guide-sider {
  background: var(--n-color);
}

.sider-header {
  border-bottom: 1px solid var(--n-border-color);
}

.guide-content {
  padding: 0;
}

.content-inner {
  max-width: 860px;
  margin: 0 auto;
  padding: 24px 32px;
}

.guide-section {
  scroll-margin-top: 80px;
}

.code-block-wrapper {
  position: relative;
  margin: 12px 0;
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--n-border-color);
}

.code-block-wrapper :deep(.n-code) {
  margin: 0;
  padding: 16px;
  font-size: 13px;
  line-height: 1.6;
  overflow-x: auto;
}

.code-block-wrapper :deep(pre) {
  margin: 0;
}

.copy-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 1;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.copy-btn:hover {
  opacity: 1;
}

.field-table {
  margin: 12px 0;
}

.field-table th {
  white-space: nowrap;
}

.guide-list {
  padding-left: 24px;
  line-height: 2;
}

.guide-list li {
  margin-bottom: 4px;
}
</style>
