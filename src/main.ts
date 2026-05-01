import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import router from "./router";
import App from "./App.vue";
import { pluginRegistry } from "./core/plugin";
import { usePluginStore } from "./stores/plugin";
import i18n from "./core/i18n";

// Import and register built-in plugins here
import settingsModule from "./plugins/settings";
pluginRegistry.register(settingsModule);

// import skillsPlugin from "./plugins/skills";
// import promptsPlugin from "./plugins/prompts";
// import devtoolsPlugin from "./plugins/devtools";

// pluginRegistry.register(skillsPlugin);
// pluginRegistry.register(promptsPlugin);
// pluginRegistry.register(devtoolsPlugin);

// 插件注册后，动态添加路由（因为 router 在 import 时创建，那时插件还没注册）
for (const route of pluginRegistry.getRoutes()) {
  router.addRoute(route);
}

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(i18n);
app.use(router);
app.use(naive);

// Refresh plugin store after registration
usePluginStore().refresh();

app.mount("#app");
