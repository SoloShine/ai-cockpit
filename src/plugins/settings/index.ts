import { SettingsOutline } from "@vicons/ionicons5";
import type { CockpitPlugin, PluginHooks, PluginModule } from "@/core/plugin";
import { useSettingsStore } from "./store";
import i18n from "@/core/i18n";

import zhCN from "./i18n/zh-CN.json";
import enUS from "./i18n/en-US.json";

const plugin: CockpitPlugin = {
  id: "settings",
  name: "设置",
  icon: SettingsOutline,
  routes: [
    {
      path: "/settings",
      name: "settings",
      component: () => import("./views/SettingsView.vue"),
      meta: { pluginId: "settings" },
    },
  ],
  navItems: [
    { routeName: "settings", label: "设置", icon: SettingsOutline },
  ],
  order: 999,
};

const hooks: PluginHooks = {
  async onInit() {
    i18n.global.mergeLocaleMessage("zh-CN", zhCN);
    i18n.global.mergeLocaleMessage("en-US", enUS);

    const store = useSettingsStore();
    await store.load();

    i18n.global.locale.value = store.appearance.language;
  },
};

const settingsModule: PluginModule = {
  default: plugin,
  hooks,
};

export default settingsModule;
