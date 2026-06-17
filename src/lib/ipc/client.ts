import { invoke } from "@tauri-apps/api/core";
import type { BootstrapPayload } from "../types";

const mockPayload: BootstrapPayload = {
  health: {
    profile: {
      id: "default",
      name: "Default Workspace",
      enabledCategories: ["core", "data", "clipboard", "system"]
    },
    commandCount: 4,
    trayReady: false,
    storageReady: false
  },
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
