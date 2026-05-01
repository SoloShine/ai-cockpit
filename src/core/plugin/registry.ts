import type { RouteRecordRaw } from "vue-router";
import type { CockpitPlugin, PluginHooks, PluginModule } from "./types";
import { ref } from "vue";

class PluginRegistry {
  private plugins = new Map<string, CockpitPlugin>();
  private hooks = new Map<string, PluginHooks>();
  private activePluginId = ref<string | null>(null);

  /**
   * Register a plugin module.
   * Resolves dependencies, validates uniqueness, and collects routes/nav.
   */
  register(module: PluginModule): void {
    const plugin = module.default;
    const hooks = module.hooks ?? {};

    if (this.plugins.has(plugin.id)) {
      console.warn(`[PluginRegistry] Plugin "${plugin.id}" already registered, skipping.`);
      return;
    }

    // Check dependencies
    if (plugin.dependsOn) {
      for (const dep of plugin.dependsOn) {
        if (!this.plugins.has(dep)) {
          console.error(
            `[PluginRegistry] Plugin "${plugin.id}" depends on "${dep}" which is not registered.`
          );
          return;
        }
      }
    }

    this.plugins.set(plugin.id, plugin);
    this.hooks.set(plugin.id, hooks);
    console.info(`[PluginRegistry] Registered plugin: ${plugin.id}`);
  }

  /** Get all registered plugins sorted by order. */
  getAll(): CockpitPlugin[] {
    return [...this.plugins.values()].sort(
      (a, b) => (a.order ?? 100) - (b.order ?? 100)
    );
  }

  /** Get a specific plugin by ID. */
  get(id: string): CockpitPlugin | undefined {
    return this.plugins.get(id);
  }

  /** Get hooks for a plugin. */
  getHooks(id: string): PluginHooks | undefined {
    return this.hooks.get(id);
  }

  /** Collect all routes from all plugins. */
  getRoutes(): RouteRecordRaw[] {
    const routes: RouteRecordRaw[] = [];
    for (const plugin of this.getAll()) {
      routes.push(...plugin.routes);
    }
    return routes;
  }

  /** Collect all nav items from all plugins, grouped by plugin. */
  getNavItems(): { pluginId: string; items: CockpitPlugin["navItems"] }[] {
    return this.getAll().map((p) => ({
      pluginId: p.id,
      items: p.navItems,
    }));
  }

  /** Set the currently active plugin (for highlighting in sidebar). */
  setActivePlugin(id: string | null): void {
    const prev = this.activePluginId.value;
    if (prev && prev !== id) {
      this.hooks.get(prev)?.onDeactivate?.();
    }
    this.activePluginId.value = id;
    if (id) {
      this.hooks.get(id)?.onActivate?.();
    }
  }

  /** Currently active plugin ID. */
  get activeId() {
    return this.activePluginId;
  }
}

/** Singleton registry instance. */
export const pluginRegistry = new PluginRegistry();
