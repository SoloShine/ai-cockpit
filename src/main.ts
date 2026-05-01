import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import router from "./router";
import App from "./App.vue";
import { pluginRegistry } from "./core/plugin";
import { usePluginStore } from "./stores/plugin";
import i18n from "./core/i18n";

// 注册内置插件
import settingsModule from "./plugins/settings";

async function bootstrap() {
  pluginRegistry.register(settingsModule);

  // 插件注册后，动态添加路由
  for (const route of pluginRegistry.getRoutes()) {
    router.addRoute(route);
  }

  // 调用所有插件的 onInit 钩子（合并 i18n 消息、加载设置等）
  for (const plugin of pluginRegistry.getAll()) {
    const hooks = pluginRegistry.getHooks(plugin.id);
    if (hooks?.onInit) {
      await hooks.onInit();
    }
  }

  const app = createApp(App);
  const pinia = createPinia();

  app.use(pinia);
  app.use(i18n);
  app.use(router);
  app.use(naive);

  usePluginStore().refresh();

  app.mount("#app");
}

bootstrap();
