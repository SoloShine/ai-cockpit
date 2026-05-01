<script setup lang="ts">
import { computed } from "vue";
import { NConfigProvider, darkTheme, zhCN, dateZhCN, enUS, dateEnUS, useOsTheme } from "naive-ui";
import AppLayout from "@/core/layout/AppLayout.vue";
import { useSettingsStore } from "@/plugins/settings/store";

const settingsStore = useSettingsStore();
const osTheme = useOsTheme();

const theme = computed(() => {
  const pref = settingsStore.appearance.theme;
  if (pref === "dark") return darkTheme;
  if (pref === "system" && osTheme.value === "dark") return darkTheme;
  return null;
});

const localeMap: Record<string, { locale: any; dateLocale: any }> = {
  "zh-CN": { locale: zhCN, dateLocale: dateZhCN },
  "en-US": { locale: enUS, dateLocale: dateEnUS },
};

const naiveLocale = computed(() => localeMap[settingsStore.appearance.language]?.locale);
const naiveDateLocale = computed(() => localeMap[settingsStore.appearance.language]?.dateLocale);
</script>

<template>
  <NConfigProvider :theme="theme" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <AppLayout />
  </NConfigProvider>
</template>
