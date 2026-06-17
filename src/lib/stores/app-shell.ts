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
  commandUsage: [],
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
  ([$appState, $query]) => rankCommands($appState.commands, $appState.commandUsage, $query)
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
    appState.update((state) => ({
      ...state,
      recentHistory,
      commandUsage: incrementCommandUsage(state.commandUsage, command.id)
    }));
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

function rankCommands(
  commands: CommandAction[],
  usageEntries: BootstrapPayload["commandUsage"],
  rawQuery: string
) {
  const queryValue = rawQuery.trim().toLowerCase();
  const usageMap = new Map(usageEntries.map((entry) => [entry.commandId, entry.executionCount]));

  if (!queryValue) {
    return [...commands].sort((left, right) => {
      const usageDelta = (usageMap.get(right.id) ?? 0) - (usageMap.get(left.id) ?? 0);
      if (usageDelta !== 0) return usageDelta;
      return left.title.localeCompare(right.title);
    });
  }

  return [...commands]
    .map((command) => ({
      command,
      score: scoreCommand(command, queryValue, usageMap.get(command.id) ?? 0)
    }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) => right.score - left.score)
    .map((entry) => entry.command);
}

function scoreCommand(command: CommandAction, queryValue: string, usageCount: number) {
  const title = command.title.toLowerCase();
  const subtitle = command.subtitle.toLowerCase();
  const tagText = command.tags.join(" ").toLowerCase();

  let score = 0;

  if (title === queryValue) score += 1000;
  else if (title.replaceAll(" ", "") === queryValue.replaceAll(" ", "")) score += 920;
  else if (title.startsWith(queryValue)) score += 760;
  else if (title.includes(queryValue)) score += 620;

  const titleFuzzy = subsequenceScore(title, queryValue);
  if (titleFuzzy > 0) score += titleFuzzy;

  for (const tag of command.tags) {
    const lowerTag = tag.toLowerCase();
    if (lowerTag === queryValue) score += 220;
    else if (lowerTag.startsWith(queryValue)) score += 150;
    else if (lowerTag.includes(queryValue)) score += 110;
  }

  const tagFuzzy = subsequenceScore(tagText, queryValue);
  if (tagFuzzy > 0) score += Math.floor(tagFuzzy * 0.35);

  if (subtitle.includes(queryValue)) score += 70;
  if (score === 0) return 0;

  score += Math.min(usageCount * 18, 180);
  return score;
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

function subsequenceScore(haystack: string, needle: string) {
  let score = 0;
  let haystackIndex = 0;
  let lastMatchIndex = -2;

  for (const char of needle) {
    const foundIndex = haystack.indexOf(char, haystackIndex);
    if (foundIndex === -1) return 0;

    score += 24;

    if (foundIndex === lastMatchIndex + 1) {
      score += 18;
    }

    if (foundIndex === 0 || haystack[foundIndex - 1] === " " || haystack[foundIndex - 1] === "-") {
      score += 12;
    }

    lastMatchIndex = foundIndex;
    haystackIndex = foundIndex + 1;
  }

  score -= Math.max(haystack.length - needle.length, 0);
  return Math.max(score, 1);
}

function incrementCommandUsage(
  usageEntries: BootstrapPayload["commandUsage"],
  commandId: string
): BootstrapPayload["commandUsage"] {
  const existing = usageEntries.find((entry) => entry.commandId === commandId);
  if (!existing) {
    return [...usageEntries, { commandId, executionCount: 1 }];
  }

  return usageEntries.map((entry) =>
    entry.commandId === commandId
      ? { ...entry, executionCount: entry.executionCount + 1 }
      : entry
  );
}
