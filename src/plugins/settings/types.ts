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
  globalPath: string;
  projectPath: string;
  enabled: boolean;
  isCustom: boolean;
}

export interface PluginSettings {
  disabledIds: string[];
  order: string[];
}

export interface RepoConfig {
  id: string;
  name: string;
  url: string;
  cachePath: string;
  enabled: boolean;
}

export interface AppSettings {
  appearance: AppearanceSettings;
  agents: AgentConfig[];
  repos: RepoConfig[];
  plugins: PluginSettings;
  _meta: {
    version: number;
    updatedAt: string;
  };
}
