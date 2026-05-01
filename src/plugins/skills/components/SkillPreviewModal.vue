<template>
  <n-modal
    :show="props.show"
    :mask-closable="true"
    @update:show="(v) => !v && emit('close')"
  >
    <n-card
      style="width: 90vw; max-width: 1200px"
      :title="t('preview.title', { name: props.skillName })"
      size="small"
      closable
      @close="emit('close')"
    >
      <n-spin :show="loadingTree || loadingContent">
        <div style="display: flex; gap: 16px; height: 80vh">
          <!-- Left panel: file tree -->
          <div
            style="width: 220px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column"
          >
            <n-tree
              :data="treeData"
              :selectable="true"
              :selected-keys="selectedKeys"
              @update:selected-keys="handleSelectKeysUpdate"
              :node-props="nodeProps"
              :expand-on-click="false"
              key-field="key"
              children-field="children"
            />
          </div>

          <!-- Right panel: content viewer -->
          <div style="flex: 1; overflow: hidden; display: flex; flex-direction: column">
            <n-scrollbar>
              <div v-if="currentFilePath" style="margin-bottom: 8px; font-weight: 500">
                {{ currentFilePath }}
              </div>
              <div
                v-if="renderedContent"
                class="content-display"
                v-html="renderedContent"
              />
              <div
                v-else
                style="flex: 1; display: flex; align-items: center; justify-content: center; color: #999"
              >
                {{ t('preview.selectFile') }}
              </div>
            </n-scrollbar>
          </div>
        </div>
      </n-spin>
    </n-card>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { NModal, NCard, NSpin, NTree, NScrollbar, type TreeOption } from "naive-ui";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import hljs from "highlight.js";
import type { FileEntry } from "../types";

const props = defineProps<{
  show: boolean;
  skillPath: string;
  skillName: string;
}>();

const emit = defineEmits<{ close: [] }>();

const { t } = useI18n();

const fileTree = ref<FileEntry[]>([]);
const selectedKeys = ref<string[]>([]);
const currentContent = ref<string>("");
const currentFilePath = ref<string>("");
const currentFileType = ref<"markdown" | "code" | "text">("text");
const loadingTree = ref(false);
const loadingContent = ref(false);

// Extension to highlight.js language mapping
const extToLang: Record<string, string> = {
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  jsx: "javascript",
  json: "json",
  rs: "rust",
  yaml: "yaml",
  yml: "yaml",
  py: "python",
  vue: "html",
  html: "html",
  css: "css",
  scss: "scss",
  xml: "xml",
  toml: "toml",
  md: "markdown",
  txt: "plaintext",
};

// Get file icon based on extension
function getFileIcon(filename: string): string {
  if (filename.endsWith(".md")) return "📄";
  if (filename.endsWith(".ts") || filename.endsWith(".js")) return "📜";
  if (filename.endsWith(".json")) return "📋";
  if (filename.endsWith(".rs")) return "⚙️";
  if (filename.endsWith(".yaml") || filename.endsWith(".yml")) return "📝";
  return "📄";
}

// Convert FileEntry to TreeOption
function convertToTreeOption(entry: FileEntry): TreeOption {
  return {
    key: entry.path,
    label: entry.name,
    children: entry.children?.map(convertToTreeOption),
    isDir: entry.isDir,
    // Add icon prefix
    prefix: () => (entry.isDir ? "📁 " : getFileIcon(entry.name) + " "),
  };
}

// Computed tree data
const treeData = computed<TreeOption[]>(() => {
  return fileTree.value.map(convertToTreeOption);
});

// Find SKILL.md in tree recursively
function findSkillMd(entries: FileEntry[]): string | null {
  for (const entry of entries) {
    if (entry.name === "SKILL.md" && !entry.isDir) {
      return entry.path;
    }
    if (entry.isDir && entry.children) {
      const found = findSkillMd(entry.children);
      if (found) return found;
    }
  }
  return null;
}

// Load file tree
async function loadFileTree() {
  if (!props.skillPath) return;

  loadingTree.value = true;
  try {
    fileTree.value = await invoke<FileEntry[]>("get_skill_file_tree", {
      skillPath: props.skillPath,
    });
  } catch (e) {
    console.error("[SkillPreviewModal] Failed to load file tree:", e);
  } finally {
    loadingTree.value = false;
  }
}

// Load file content
async function loadFileContent(filePath: string) {
  loadingContent.value = true;
  currentFilePath.value = filePath;
  currentContent.value = "";

  try {
    const content = await invoke<string>("read_skill_file", { filePath });

    // Determine file type
    const ext = filePath.split(".").pop()?.toLowerCase() || "";
    if (ext === "md") {
      currentFileType.value = "markdown";
      const result = marked(content, { gfm: true, breaks: true });
      currentContent.value = typeof result === "string" ? result : "";
    } else {
      currentFileType.value = "code";
      const lang = extToLang[ext] || "plaintext";
      try {
        const highlighted = hljs.highlight(content, { language: lang });
        currentContent.value = highlighted.value;
      } catch {
        // Fallback to escaped HTML
        currentFileType.value = "text";
        currentContent.value = escapeHtml(content);
      }
    }
  } catch (e) {
    console.error("[SkillPreviewModal] Failed to load file content:", e);
    currentContent.value = `<span style="color: red">${t("preview.loadError")}</span>`;
  } finally {
    loadingContent.value = false;
  }
}

// Escape HTML for text display
function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// Get language from file path
function getLanguageFromPath(filePath: string): string {
  const ext = filePath.split(".").pop()?.toLowerCase() || "";
  return extToLang[ext] || "plaintext";
}

// Computed rendered content
const renderedContent = computed(() => {
  if (currentFileType.value === "code") {
    return `<pre><code class="hljs language-${getLanguageFromPath(currentFilePath.value)}">${currentContent.value}</code></pre>`;
  }
  if (currentFileType.value === "text") {
    return `<pre style="white-space: pre-wrap; word-break: break-all;">${currentContent.value}</pre>`;
  }
  return currentContent.value;
});

// Handle file selection
function handleSelectKeysUpdate(keys: string[]) {
  selectedKeys.value = keys;
  if (keys.length > 0) {
    const filePath = keys[0];
    // Only load files, not directories
    const entry = findEntryByPath(fileTree.value, filePath);
    if (entry && !entry.isDir) {
      loadFileContent(filePath);
    }
  }
}

// Find entry by path in tree
function findEntryByPath(entries: FileEntry[], path: string): FileEntry | null {
  for (const entry of entries) {
    if (entry.path === path) return entry;
    if (entry.isDir && entry.children) {
      const found = findEntryByPath(entry.children, path);
      if (found) return found;
    }
  }
  return null;
}

// Node props for tree
const nodeProps = () => {
  return {
    style: {
      cursor: "pointer",
    },
  };
};

// Watch for modal open
watch(
  () => props.show,
  async (show) => {
    if (show && props.skillPath) {
      // Reset state
      selectedKeys.value = [];
      currentContent.value = "";
      currentFilePath.value = "";
      currentFileType.value = "text";

      // Load file tree
      await loadFileTree();

      // Auto-select SKILL.md if exists
      const skillMdPath = findSkillMd(fileTree.value);
      if (skillMdPath) {
        handleSelectKeysUpdate([skillMdPath]);
      }
    }
  }
);
</script>

<style scoped>
.content-display {
  padding: 12px;
  line-height: 1.6;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* Markdown styling */
.content-display :deep(h1),
.content-display :deep(h2),
.content-display :deep(h3),
.content-display :deep(h4),
.content-display :deep(h5),
.content-display :deep(h6) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

.content-display :deep(h1) {
  font-size: 2em;
  border-bottom: 1px solid #e0e0e6;
  padding-bottom: 8px;
}

.content-display :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid #e0e0e6;
  padding-bottom: 8px;
}

.content-display :deep(p) {
  margin-bottom: 16px;
}

.content-display :deep(code) {
  background-color: #f6f8fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  font-size: 0.9em;
}

.content-display :deep(pre) {
  background-color: #f6f8fa;
  padding: 16px;
  border-radius: 6px;
  overflow-x: auto;
  margin-bottom: 16px;
}

.content-display :deep(pre code) {
  background-color: transparent;
  padding: 0;
  border-radius: 0;
}

.content-display :deep(ul),
.content-display :deep(ol) {
  padding-left: 24px;
  margin-bottom: 16px;
}

.content-display :deep(li) {
  margin-bottom: 4px;
}

.content-display :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin-bottom: 16px;
}

.content-display :deep(table th),
.content-display :deep(table td) {
  border: 1px solid #d0d7de;
  padding: 6px 13px;
}

.content-display :deep(table th) {
  background-color: #f6f8fa;
  font-weight: 600;
}

.content-display :deep(blockquote) {
  border-left: 4px solid #d0d7de;
  padding-left: 16px;
  color: #656d76;
  margin-bottom: 16px;
}

.content-display :deep(a) {
  color: #0969da;
  text-decoration: none;
}

.content-display :deep(a:hover) {
  text-decoration: underline;
}

.content-display :deep(img) {
  max-width: 100%;
  height: auto;
}

/* Highlight.js integration */
.content-display :deep(.hljs) {
  background: transparent;
  padding: 0;
}

.content-display :deep(pre code.hljs) {
  background: #f6f8fa;
  padding: 16px;
  border-radius: 6px;
  display: block;
  overflow-x: auto;
}
</style>
