import { createRouter, createWebHistory } from "vue-router";
import { pluginRegistry } from "@/core/plugin";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/welcome",
    },
    {
      path: "/welcome",
      name: "welcome",
      component: () => import("@/views/WelcomeView.vue"),
      meta: { title: "AI Cockpit" },
    },
    ...pluginRegistry.getRoutes(),
  ],
});

// Track active plugin based on current route
router.afterEach((to) => {
  const pluginId = to.meta?.pluginId as string | undefined;
  pluginRegistry.setActivePlugin(pluginId ?? null);
});

export default router;
