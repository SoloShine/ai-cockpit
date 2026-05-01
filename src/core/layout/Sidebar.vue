<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { usePluginStore } from "@/stores/plugin";
import { useSettingsStore } from "@/plugins/settings/store";
import { NLayoutSider, NMenu, NIcon, NDivider, NText } from "naive-ui";
import type { MenuOption } from "naive-ui";
import {
  RocketOutline,
  SettingsOutline,
} from "@vicons/ionicons5";
import type { Component } from "vue";

const router = useRouter();
const pluginStore = usePluginStore();
const settingsStore = useSettingsStore();

function renderIcon(icon: string | Component): (() => Component) | undefined {
  if (typeof icon === "string") return undefined;
  return () => icon;
}

const pluginMenuOptions = computed<MenuOption[]>(() =>
  pluginStore.navGroups
    .filter(({ pluginId }) => !settingsStore.plugins.disabledIds.includes(pluginId))
    .map(({ pluginId, items }) => {
    if (items.length === 1) {
      const item = items[0];
      return {
        label: item.label,
        key: item.routeName,
        icon: item.icon ? renderIcon(item.icon) : undefined,
      };
    }
    return {
      label: pluginStore.plugins.find((p) => p.id === pluginId)?.name ?? pluginId,
      key: pluginId,
      icon: renderIcon(
        pluginStore.plugins.find((p) => p.id === pluginId)?.icon ?? ""
      ),
      children: items.map((item) => ({
        label: item.label,
        key: item.routeName,
        icon: item.icon ? renderIcon(item.icon) : undefined,
      })),
    };
  })
);

const fixedMenuOptions: MenuOption[] = [
  {
    type: "divider",
    key: "d-settings",
  },
  {
    label: "设置",
    key: "settings",
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
];

import { h } from "vue";

const allMenuOptions = computed(() => [
  ...pluginMenuOptions.value,
  ...fixedMenuOptions,
]);

function handleMenuUpdate(key: string) {
  if (key === "settings") {
    router.push({ name: "settings" });
  } else {
    router.push({ name: key });
  }
}

// Compute active key from current route
const activeKey = computed(() => {
  const route = router.currentRoute.value;
  return (route.name as string) ?? null;
});
</script>

<template>
  <NLayoutSider
    bordered
    :width="220"
    :native-scrollbar="false"
    content-style="padding: 8px;"
  >
    <div class="logo" @click="router.push('/')">
      <NIcon size="24"><RocketOutline /></NIcon>
      <NText strong style="margin-left: 8px; font-size: 16px">AI Cockpit</NText>
    </div>
    <NDivider style="margin: 8px 0" />
    <NMenu
      :options="allMenuOptions"
      :value="activeKey"
      @update:value="handleMenuUpdate"
    />
  </NLayoutSider>
</template>

<style scoped>
.logo {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
}
</style>
