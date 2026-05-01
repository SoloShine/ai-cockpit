// src/plugins/skills/index.ts
import { RocketOutline } from "@vicons/ionicons5";
import type { PluginModule } from "@/core/plugin";
import { useSkillsStore } from "./store";
import i18n from "@/core/i18n";

import zhCN from "./i18n/zh-CN.json";
import enUS from "./i18n/en-US.json";

const plugin: PluginModule = {
  default: {
    id: "skills",
    name: "Skill 管理",
    icon: RocketOutline,
    routes: [
      {
        path: "/skills",
        name: "skills",
        component: () => import("./views/SkillsMainView.vue"),
        meta: { pluginId: "skills" },
      },
    ],
    navItems: [
      {
        routeName: "skills",
        label: "Skill 管理",
        icon: RocketOutline,
      },
    ],
    order: 10,
  },
  hooks: {
    async onInit() {
      i18n.global.mergeLocaleMessage("zh-CN", zhCN);
      i18n.global.mergeLocaleMessage("en-US", enUS);
    },
    async onActivate() {
      const store = useSkillsStore();
      await store.scanAllAgents(store.currentScope);
    },
  },
};

export default plugin;
