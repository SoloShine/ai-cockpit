import { createApp } from "vue";
import { createPinia } from "pinia";
import naive from "naive-ui";
import router from "./router";
import App from "./App.vue";
import { pluginRegistry } from "./core/plugin";
import { usePluginStore } from "./stores/plugin";

// Import and register built-in plugins here
// import skillsPlugin from "./plugins/skills";
// import promptsPlugin from "./plugins/prompts";
// import devtoolsPlugin from "./plugins/devtools";

// pluginRegistry.register(skillsPlugin);
// pluginRegistry.register(promptsPlugin);
// pluginRegistry.register(devtoolsPlugin);

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(naive);

// Refresh plugin store after registration
usePluginStore().refresh();

app.mount("#app");
