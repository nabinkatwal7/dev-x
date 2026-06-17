import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, BootstrapPayload, CommandHistoryEntry, WorkspaceProfile } from "../types";

const mockPayload: BootstrapPayload = {
  health: {
    profile: {
      id: "default",
      name: "Default Workspace",
      enabledCategories: ["core", "data", "clipboard", "system"],
      isDefault: true
    },
    commandCount: 4,
    trayReady: false,
    storageReady: false
  },
  settings: {
    themeMode: "system",
    launchHotkey: "Alt+Space",
    closeToTray: false,
    historyLimit: 50
  },
  profiles: [
    {
      id: "default",
      name: "Default Workspace",
      enabledCategories: ["core", "data", "clipboard", "system"],
      isDefault: true
    }
  ],
  recentHistory: [],
  commands: [
    {
      id: "core.open-command-palette",
      title: "Open Command Palette",
      subtitle: "Primary keyboard-first launcher overlay.",
      category: "core",
      tags: ["launcher", "overlay", "omnibar"],
      shortcut: "Alt+Space"
    },
    {
      id: "data.format-json",
      title: "Format JSON",
      subtitle: "Pretty-print or minify JSON payloads locally.",
      category: "data",
      tags: ["json", "formatter", "payload"]
    },
    {
      id: "clipboard.history",
      title: "Clipboard History",
      subtitle: "Inspect and replay structured clipboard items.",
      category: "clipboard",
      tags: ["clipboard", "history", "stack"]
    },
    {
      id: "system.port-monitor",
      title: "Port Monitor",
      subtitle: "Track listening local ports and owning processes.",
      category: "system",
      tags: ["ports", "process", "network"]
    }
  ]
};

export async function bootstrapApp(): Promise<BootstrapPayload> {
  try {
    return await invoke<BootstrapPayload>("bootstrap_app");
  } catch {
    return mockPayload;
  }
}

export async function updateAppSettings(settings: AppSettings) {
  return invoke<AppSettings>("update_app_settings", { payload: settings });
}

export async function recordCommandExecution(commandId: string, queryText: string) {
  return invoke<CommandHistoryEntry[]>("record_command_execution", {
    payload: { commandId, queryText }
  });
}

export async function setActiveProfile(profileId: string) {
  return invoke<WorkspaceProfile>("set_active_profile", { profileId });
}
