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
  localCount: number;
  sameCount: number;
  outdatedCount: number;
  remoteOnlyCount: number;
  lastModified?: string;
  readmePreview?: string;
}

/** Result of syncing a single repository */
export interface SyncResult {
  repoId: string
  success: boolean
  message: string
  skillCount: number
}

/** Summary of a skill in a remote repository */
export interface RemoteSkillInfo {
  name: string
  description: string
  version?: string
  sourceRepo: string
  skillType: 'file' | 'directory'
}

/** Detailed info about a remote skill */
export interface RemoteSkillDetail {
  info: RemoteSkillInfo
  files: FileEntry[]
  contentHash: string
}

/** Status of a skill comparison between local and remote */
export type ComparisonStatus = 'same' | 'outdated' | 'localOnly' | 'remoteOnly'

/** A comparison pairing local and remote skill info */
export interface SkillComparison {
  name: string
  status: ComparisonStatus
  local?: SkillInfo
  remote?: SkillInfo
  sourceRepo?: string
}

/** File-level diff entry */
export interface FileDiffEntry {
  path: string
  fileName: string
  diffType: DiffStatus
  localSize?: number
  remoteSize?: number
}

/** Full skill diff result */
export interface SkillDiffResult {
  skillName: string
  fileDiffs: FileDiffEntry[]
  addedCount: number
  removedCount: number
  modifiedCount: number
  unchangedCount: number
}

/** File content for line-by-line diff */
export interface DiffFileContent {
  localContent?: string
  remoteContent?: string
}

/** A single line in a diff output */
export interface DiffLine {
  type: 'added' | 'removed' | 'unchanged'
  oldLineNumber?: number
  newLineNumber?: number
  content: string
}

/** A single operation record in the history log */
export interface OperationRecord {
  id: string
  operationType: 'install' | 'update' | 'uninstall'
  skillName: string
  targetPath: string
  sourcePath?: string
  timestamp: string
  versionBefore?: string
  versionAfter?: string
  canRollback: boolean
  rolledBack: boolean
}

/** Migration scan result for a single skill */
export interface MigrateSkillItem {
  name: string
  sourcePath: string
  targetPath: string
  status: 'newTarget' | 'sameContent' | 'differentVersion' | 'contentDiffers'
  sourceHash?: string
  targetHash?: string
  version?: string
  description?: string
}

/** Migration conflict resolution */
export type ConflictResolution = 'Skip' | 'Overwrite'

/** Migration result summary */
export interface MigrateResult {
  migrated: string[]
  skipped: string[]
  failed: { name: string; error: string }[]
}