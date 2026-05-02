// src/plugins/skills/index.ts
import { RocketOutline, FolderOutline, BookOutline } from "@vicons/ionicons5";
import type { PluginModule } from "@/core/plugin";
import i18n from "@/core/i18n";
import RepoPanel from "./components/RepoPanel.vue";

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
      {
        path: "/skills/projects",
        name: "skills-projects",
        component: () => import("./views/ProjectListView.vue"),
        meta: { pluginId: "skills" },
      },
      {
        path: "/skills/projects/:encodedPath",
        name: "skills-project-detail",
        component: () => import("./views/ProjectDetailView.vue"),
        meta: { pluginId: "skills" },
      },
      {
        path: "/skills/guide",
        name: "skills-guide",
        component: () => import("./views/GuideView.vue"),
        meta: { pluginId: "skills" },
      },
    ],
    navItems: [
      {
        routeName: "skills",
        label: "全局 Skills",
        icon: RocketOutline,
      },
      {
        routeName: "skills-projects",
        label: "项目 Skills",
        icon: FolderOutline,
      },
      {
        routeName: "skills-guide",
        label: "开发指南",
        icon: BookOutline,
      },
    ],
    order: 10,
  },
  hooks: {
    async onInit() {
      i18n.global.mergeLocaleMessage("zh-CN", zhCN);
      i18n.global.mergeLocaleMessage("en-US", enUS);
    },
    async onActivate() {},
    SettingsPanel: RepoPanel,
  },
};

export default plugin;
