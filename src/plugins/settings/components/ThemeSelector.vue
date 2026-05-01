<script setup lang="ts">
import { NCard, NIcon, NText } from "naive-ui";
import { SunnyOutline, MoonOutline, DesktopOutline } from "@vicons/ionicons5";
import { useI18n } from "vue-i18n";
import type { AppearanceSettings } from "../types";

defineProps<{ modelValue: AppearanceSettings["theme"] }>();
const emit = defineEmits<{ "update:modelValue": [value: AppearanceSettings["theme"]] }>();
const { t } = useI18n();

const themes: { value: AppearanceSettings["theme"]; icon: any; label: string }[] = [
  { value: "light", icon: SunnyOutline, label: t("settings.appearance.themeLight") },
  { value: "dark", icon: MoonOutline, label: t("settings.appearance.themeDark") },
  { value: "system", icon: DesktopOutline, label: t("settings.appearance.themeSystem") },
];
</script>

<template>
  <div style="display: flex; gap: 12px">
    <NCard
      v-for="theme in themes"
      :key="theme.value"
      hoverable
      :class="{ 'theme-card--active': modelValue === theme.value }"
      style="flex: 1; cursor: pointer; text-align: center"
      @click="emit('update:modelValue', theme.value)"
    >
      <NIcon size="32"><component :is="theme.icon" /></NIcon>
      <div style="margin-top: 8px">
        <NText>{{ theme.label }}</NText>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.theme-card--active {
  outline: 2px solid var(--n-border-color);
  outline-offset: -2px;
}
</style>
