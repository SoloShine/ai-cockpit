<script setup lang="ts">
import { computed } from "vue";
import { NSpace } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../store";
import type { ComparisonStatus } from "../types";

const { t } = useI18n();
const store = useSkillsStore();

type FilterValue = ComparisonStatus | null;

const chips = computed(() => [
  { status: null as FilterValue, label: t("skills.filter.all"), count: store.comparisons.length, cls: "chip-all" },
  { status: "same" as FilterValue, label: t("skills.filter.same"), count: store.comparisonCounts.same, cls: "chip-same" },
  { status: "outdated" as FilterValue, label: t("skills.filter.outdated"), count: store.comparisonCounts.outdated, cls: "chip-outdated" },
  { status: "localOnly" as FilterValue, label: t("skills.filter.localOnly"), count: store.comparisonCounts.localOnly, cls: "chip-local" },
  { status: "remoteOnly" as FilterValue, label: t("skills.filter.remoteOnly"), count: store.comparisonCounts.remoteOnly, cls: "chip-remote" },
]);

function toggle(status: FilterValue) {
  store.statusFilter = store.statusFilter === status ? null : status;
}
</script>

<template>
  <NSpace :size="4" v-if="store.comparisons.length > 0" align="center" data-testid="status-filter-bar">
    <button
      v-for="chip in chips"
      :key="String(chip.status)"
      class="status-chip"
      :class="[chip.cls, { active: store.statusFilter === chip.status }]"
      @click="toggle(chip.status)"
    >
      {{ chip.label }}
      <span class="chip-count">{{ chip.count }}</span>
    </button>
  </NSpace>
</template>

<style scoped>
.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 12px;
  border-radius: 14px;
  border: 1px solid var(--n-border-color, #e0e0e6);
  background: transparent;
  color: var(--n-text-color, #333);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  line-height: 1.4;
}
.status-chip:hover {
  border-color: var(--n-primary-color, #18a058);
  color: var(--n-primary-color, #18a058);
}
.status-chip.active {
  color: #fff;
  border-color: var(--n-primary-color, #18a058);
  background: var(--n-primary-color, #18a058);
}
.chip-same.active {
  background: #18a058;
  border-color: #18a058;
}
.chip-outdated.active {
  background: #f0a020;
  border-color: #f0a020;
}
.chip-local.active {
  background: #2080f0;
  border-color: #2080f0;
}
.chip-remote.active {
  background: #d03050;
  border-color: #d03050;
}
.chip-count {
  font-weight: 600;
  font-size: 12px;
}
</style>
