export interface SkillMeta {
  name: string;
  description: string;
  version?: string;
  author?: string;
  tags: string[];
  dependencies: string[];
}

export interface SkillInfo {
  name: string;
  path: string;
  isFile: boolean;
  hasSkillMd: boolean;
  meta?: SkillMeta;
  fileCount: number;
  sizeBytes: number;
  contentHash: string;
  lastModified?: string;
  sourceAgentId?: string;
}

export type SkillScope = "global" | "project";

export interface ScanResult {
  agentId: string;
  scope: SkillScope;
  skills: SkillInfo[];
  total: number;
}

export interface FileEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  children: FileEntry[];
}

export type DiffStatus = "same" | "modified" | "added" | "removed";

export interface OperationResult {
  success: boolean;
  message: string;
  affectedPaths: string[];
}

export type OperationType = "install" | "update" | "uninstall";

export interface SkillOperation {
  operationType: OperationType;
  source: string;
  targetPath: string;
}

export interface ProjectOverview {
  projectPath: string;
  projectName: string;
  agentSkillsCount: Record<string, number>;
}
