// src/plugins/skills/composables/useColumnResize.ts
import { ref, watch } from "vue";

const STORAGE_PREFIX = "col_resize_";
const DEBOUNCE_MS = 200;
const MIN_WIDTH = 60;
const MAX_WIDTH = 500;

export function useColumnResize(
  tableId: string,
  defaultWidths: Record<string, number>,
) {
  // Load persisted widths
  const storageKey = STORAGE_PREFIX + tableId;
  let saved: Record<string, number> = {};
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw) saved = JSON.parse(raw);
  } catch {
    // Ignore parse errors
  }

  // Merge saved with defaults
  const merged: Record<string, number> = { ...defaultWidths };
  for (const [key, val] of Object.entries(saved)) {
    if (key in merged) {
      merged[key] = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, val));
    }
  }

  const columnWidths = ref<Record<string, number>>(merged);

  // Debounced persistence
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  watch(
    columnWidths,
    (val) => {
      if (saveTimer) clearTimeout(saveTimer);
      saveTimer = setTimeout(() => {
        try {
          localStorage.setItem(storageKey, JSON.stringify(val));
        } catch {
          // Ignore storage errors
        }
      }, DEBOUNCE_MS);
    },
    { deep: true },
  );

  function getColumnWidth(key: string): number {
    return columnWidths.value[key] ?? defaultWidths[key] ?? 120;
  }

  /** Create a mousedown handler for a column resize handle */
  function handleResizeMousedown(key: string, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();

    const startX = event.clientX;
    const startWidth = getColumnWidth(key);

    function onMouseMove(e: MouseEvent) {
      const delta = e.clientX - startX;
      const newWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, startWidth + delta));
      columnWidths.value = { ...columnWidths.value, [key]: newWidth };
    }

    function onMouseUp() {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }

  return {
    columnWidths,
    getColumnWidth,
    handleResizeMousedown,
  };
}
