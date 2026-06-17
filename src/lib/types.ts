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
}

export interface AppHealth {
  profile: WorkspaceProfile;
  commandCount: number;
  trayReady: boolean;
  storageReady: boolean;
}

export interface BootstrapPayload {
  health: AppHealth;
  commands: CommandAction[];
}
