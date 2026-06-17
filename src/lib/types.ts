export type CommandCategory =
  | "core"
  | "clipboard"
  | "data"
  | "system"
  | "crypto"
  | "network"
  | "filesystem"
  | "ai";

export interface CommandAction {
  id: string;
  title: string;
  subtitle: string;
  category: CommandCategory;
  tags: string[];
  shortcut?: string;
  acceptsInput: boolean;
}

export interface CommandPreview {
  title: string;
  body: string;
  hints: string[];
}

export interface WorkspaceProfile {
  id: string;
  name: string;
  environmentTags: string[];
  enabledCategories: CommandCategory[];
  enabledCommandIds: string[];
  defaultCommandId: string;
  launchHotkey: string;
  isDefault: boolean;
}

export interface AppHealth {
  profile: WorkspaceProfile;
  commandCount: number;
  trayReady: boolean;
  storageReady: boolean;
  extensionDirectory: string;
}

export interface AppSettings {
  themeMode: string;
  launchHotkey: string;
  closeToTray: boolean;
  historyLimit: number;
}

export interface CommandHistoryEntry {
  id: number;
  commandId: string;
  queryText: string;
  inputText: string;
  executedAt: string;
}

export interface CommandUsageEntry {
  commandId: string;
  executionCount: number;
}

export type CommandExecutionStatus = "success" | "error" | "info";

export interface CommandExecutionResult {
  commandId: string;
  title: string;
  output: string;
  status: CommandExecutionStatus;
  summary: string;
}

export interface BootstrapPayload {
  health: AppHealth;
  settings: AppSettings;
  profiles: WorkspaceProfile[];
  recentHistory: CommandHistoryEntry[];
  commandUsage: CommandUsageEntry[];
  pinnedModules: PinnedModule[];
  extensions: ScriptExtensionSummary[];
  commands: CommandAction[];
}

export interface SaveWorkspaceProfilePayload {
  id?: string;
  name: string;
  environmentTags: string[];
  enabledCommandIds: string[];
  defaultCommandId: string;
  launchHotkey: string;
}

export interface PinnedModule {
  commandId: string;
  windowLabel: string;
  title: string;
}

export interface ScriptExtensionSummary {
  id: string;
  title: string;
  subtitle: string;
  sourcePath: string;
  commandPath: string;
  acceptsInput: boolean;
}

export interface SearchHistoryPayload {
  queryText: string;
  limit: number;
}

export interface ConfigurationSnapshot {
  settings: AppSettings;
  profiles: WorkspaceProfile[];
}
