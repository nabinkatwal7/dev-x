import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BootstrapPayload,
  CommandExecutionResult,
  CommandHistoryEntry
} from "../types";

export async function bootstrapApp(): Promise<BootstrapPayload> {
  return invoke<BootstrapPayload>("bootstrap_app");
}

export async function updateAppSettings(settings: AppSettings) {
  return invoke<AppSettings>("update_app_settings", { payload: settings });
}

export async function recordCommandExecution(commandId: string, queryText: string) {
  return invoke<CommandHistoryEntry[]>("record_command_execution", {
    payload: { commandId, queryText }
  });
}

export async function executeCommand(commandId: string, input: string) {
  return invoke<CommandExecutionResult>("execute_command", {
    payload: { commandId, input }
  });
}

export async function hideOverlay() {
  return invoke<void>("hide_overlay");
}
