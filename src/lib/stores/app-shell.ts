import { derived, get, writable } from "svelte/store";
import { executeCommand, recordCommandExecution, updateAppSettings } from "../ipc/client";
import type {
  AppSettings,
  BootstrapPayload,
  CommandAction,
  CommandExecutionResult
} from "../types";

const initialState: BootstrapPayload = {
  health: {
    profile: {
      id: "loading",
      name: "Loading",
      enabledCategories: ["core"],
      isDefault: false
    },
    commandCount: 0,
    trayReady: false,
    storageReady: false
  },
  settings: {
    themeMode: "system",
    launchHotkey: "Alt+Space",
    closeToTray: false,
    historyLimit: 50
  },
  profiles: [],
  recentHistory: [],
  commands: []
};

export const appState = writable(initialState);
export const query = writable("");
export const selectedCommandId = writable<string | null>(null);
export const commandInput = writable(defaultCommandInput("data.format-json"));
export const commandResult = writable<CommandExecutionResult | null>(null);
export const commandError = writable<string | null>(null);
export const commandRunning = writable(false);

export const filteredCommands = derived(
  [appState, query],
  ([$appState, $query]) => rankCommands($appState.commands, $query)
);

export const selectedCommand = derived(
  [filteredCommands, selectedCommandId],
  ([$filteredCommands, $selectedCommandId]) =>
    $filteredCommands.find((command) => command.id === $selectedCommandId) ?? $filteredCommands[0] ?? null
);

export function loadBootstrap(payload: BootstrapPayload) {
  appState.set(payload);
  const [first] = payload.commands;
  selectedCommandId.set(first?.id ?? null);
  commandInput.set(defaultCommandInput(first?.id));
}

export async function executeSelectedCommand(command: CommandAction | null, currentQuery: string) {
  if (!command) return;

  commandRunning.set(true);
  commandError.set(null);

  try {
    const input = get(commandInput);
    const result = await executeCommand(command.id, input);
    commandResult.set(result);
    const historyQuery = currentQuery.trim() || summarizeExecutionInput(input, command.id);
    const recentHistory = await recordCommandExecution(command.id, historyQuery);
    appState.update((state) => ({ ...state, recentHistory }));
  } catch (error) {
    const message = error instanceof Error ? error.message : "Command execution failed.";
    commandError.set(message);
    appState.update((state) => ({
      ...state,
      recentHistory: [
        {
          id: Date.now(),
          commandId: command.id,
          queryText: currentQuery.trim() || summarizeExecutionInput(get(commandInput), command.id),
          executedAt: "local-fallback"
        },
        ...state.recentHistory
      ].slice(0, 10)
    }));
  } finally {
    commandRunning.set(false);
  }
}

export async function saveSettings(settings: AppSettings) {
  try {
    const saved = await updateAppSettings(settings);
    appState.update((state) => ({ ...state, settings: saved }));
  } catch {
    appState.update((state) => ({ ...state, settings }));
  }
}

export function selectCommand(command: CommandAction) {
  selectedCommandId.set(command.id);
  commandInput.set(defaultCommandInput(command.id));
  commandResult.set(null);
  commandError.set(null);
}

export function updateCommandInput(value: string) {
  commandInput.set(value);
}

function rankCommands(commands: CommandAction[], rawQuery: string) {
  const queryValue = rawQuery.trim().toLowerCase();
  if (!queryValue) {
    return commands;
  }

  return [...commands]
    .map((command) => ({
      command,
      score: scoreCommand(command, queryValue)
    }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) => right.score - left.score)
    .map((entry) => entry.command);
}

function scoreCommand(command: CommandAction, queryValue: string) {
  const title = command.title.toLowerCase();
  const subtitle = command.subtitle.toLowerCase();
  const tags = command.tags.join(" ").toLowerCase();

  if (title === queryValue) return 100;
  if (title.startsWith(queryValue)) return 80;
  if (title.includes(queryValue)) return 60;
  if (tags.includes(queryValue)) return 30;
  if (subtitle.includes(queryValue)) return 15;
  return 0;
}

function defaultCommandInput(commandId?: string | null) {
  switch (commandId) {
    case "data.format-json":
    case "data.minify-json":
      return "";
    default:
      return "";
  }
}

function summarizeExecutionInput(input: string, commandId: string) {
  const firstLine = input.trim().split("\n")[0]?.slice(0, 80);
  return firstLine || commandId;
}
