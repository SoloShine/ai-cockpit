// src/plugins/settings/types.ts
export interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "zh-CN" | "en-US";
  fontSize: number;
}

export interface AgentConfig {
  id: string;
  name: string;
  type: string;
  basePath: string;
  enabled: boolean;
  isCustom: boolean;
}

export interface PluginSettings {
  disabledIds: string[];
  order: string[];
}

export interface AppSettings {
  appearance: AppearanceSettings;
  agents: AgentConfig[];
  plugins: PluginSettings;
  _meta: {
    version: number;
    updatedAt: string;
  };
}
