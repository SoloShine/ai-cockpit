import type { RouteRecordRaw } from "vue-router";
import type { Component } from "vue";

/**
 * Plugin manifest — describes a plugin's identity and capabilities.
 * Every plugin must export a default object implementing this interface.
 */
export interface CockpitPlugin {
  /** Unique identifier, e.g. "skills", "prompts", "devtools" */
  id: string;
  /** Display name (supports i18n key) */
  name: string;
  /** Short description */
  description?: string;
  /** Sidebar icon component or icon name */
  icon: string | Component;
  /** Vue Router routes contributed by this plugin */
  routes: RouteRecordRaw[];
  /** Sidebar navigation items */
  navItems: NavItem[];
  /** Plugin priority (lower = higher in sidebar). Default 100 */
  order?: number;
  /** Required plugins that must be loaded first */
  dependsOn?: string[];
}

export interface NavItem {
  /** Route name to navigate to */
  routeName: string;
  /** Display label (i18n key or plain text) */
  label: string;
  /** Icon name or component */
  icon?: string | Component;
  /** Nested children for sub-navigation */
  children?: NavItem[];
}

/**
 * Plugin lifecycle hooks.
 * Plugins may optionally implement these to integrate with the app shell.
 */
export interface PluginHooks {
  /** Called when the plugin is registered. Good place for one-time setup. */
  onInit?: () => void | Promise<void>;
  /** Called when the plugin's routes are first navigated to. */
  onActivate?: () => void | Promise<void>;
  /** Called when navigating away from the plugin's routes. */
  onDeactivate?: () => void | Promise<void>;
  /** Settings panel component rendered in the global Settings page */
  SettingsPanel?: Component;
}

/**
 * Full plugin module shape — what each plugin file must export.
 */
export interface PluginModule {
  default: CockpitPlugin;
  hooks?: PluginHooks;
}
