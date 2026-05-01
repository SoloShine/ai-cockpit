import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import router from "./router";
import App from "./App.vue";
import { pluginRegistry } from "./core/plugin";
import { usePluginStore } from "./stores/plugin";
import i18n from "./core/i18n";

import settingsModule from "./plugins/settings";

async function bootstrap() {
  const app = createApp(App);
  const pinia = createPinia();

  // 先安装 Pinia 和 i18n，onInit 中的 store 才能工作
  app.use(pinia);
  app.use(i18n);

  // 注册插件
  pluginRegistry.register(settingsModule);

  // 动态添加路由
  for (const route of pluginRegistry.getRoutes()) {
    router.addRoute(route);
  }

  // 调用 onInit（现在 Pinia 可用，store 能正常创建）
  for (const plugin of pluginRegistry.getAll()) {
    const hooks = pluginRegistry.getHooks(plugin.id);
    if (hooks?.onInit) {
      await hooks.onInit();
    }
  }

  app.use(router);
  app.use(naive);

  usePluginStore().refresh();

  app.mount("#app");
}

bootstrap();
