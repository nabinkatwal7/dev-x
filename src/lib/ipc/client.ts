import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BootstrapPayload,
  ConfigurationSnapshot,
  CommandExecutionResult,
  CommandHistoryEntry,
  SaveWorkspaceProfilePayload
} from "../types";

export async function bootstrapApp(): Promise<BootstrapPayload> {
  return invoke<BootstrapPayload>("bootstrap_app");
}

export async function updateAppSettings(settings: AppSettings) {
  return invoke<AppSettings>("update_app_settings", { payload: settings });
}

export async function recordCommandExecution(commandId: string, queryText: string, inputText: string) {
  return invoke<CommandHistoryEntry[]>("record_command_execution", {
    payload: { commandId, queryText, inputText }
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

export async function setActiveProfile(profileId: string) {
  return invoke<BootstrapPayload>("set_active_profile", { profileId });
}

export async function saveWorkspaceProfile(payload: SaveWorkspaceProfilePayload) {
  return invoke<BootstrapPayload>("save_workspace_profile", { payload });
}

export async function searchCommandHistory(queryText: string, limit = 50) {
  return invoke<CommandHistoryEntry[]>("search_command_history", {
    payload: { queryText, limit }
  });
}

export async function togglePinnedModule(commandId: string) {
  return invoke<BootstrapPayload>("toggle_pinned_module", { commandId });
}

export async function reloadExtensions() {
  return invoke<BootstrapPayload>("reload_extensions");
}

export async function exportConfigurationSnapshot() {
  return invoke<ConfigurationSnapshot>("export_configuration_snapshot");
}

export async function importConfiguration(snapshot: ConfigurationSnapshot) {
  return invoke<BootstrapPayload>("import_configuration", { payload: { snapshot } });
}

export async function writeTextFile(path: string, contents: string) {
  return invoke<void>("write_text_file", { payload: { path, contents } });
}

export async function readTextFile(path: string) {
  return invoke<string>("read_text_file", { payload: { path } });
}
