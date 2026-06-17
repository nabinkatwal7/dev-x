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
  enabledCategories: CommandCategory[];
  isDefault: boolean;
}

export interface AppHealth {
  profile: WorkspaceProfile;
  commandCount: number;
  trayReady: boolean;
  storageReady: boolean;
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
  commands: CommandAction[];
}
