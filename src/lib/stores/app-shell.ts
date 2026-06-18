import { derived, get, writable } from "svelte/store";
import { getToolSpec } from "../core/tool-meta";
import {
  executeCommand,
  exportConfigurationSnapshot,
  importConfiguration,
  readTextFile,
  recordCommandExecution,
  reloadExtensions,
  saveWorkspaceProfile,
  searchCommandHistory,
  setActiveProfile,
  togglePinnedModule,
  updateAppSettings
} from "../ipc/client";
import type {
  AppSettings,
  BootstrapPayload,
  CommandCategory,
  CommandAction,
  CommandExecutionResult,
  CommandHistoryEntry,
  ConfigurationSnapshot,
  SaveWorkspaceProfilePayload,
  WorkspaceProfile
} from "../types";

const initialState: BootstrapPayload = {
  health: {
    profile: {
      id: "loading",
      name: "Loading",
      environmentTags: [],
      enabledCategories: ["core"],
      enabledCommandIds: [],
      defaultCommandId: "",
      launchHotkey: "Alt+Space",
      isDefault: false
    },
    commandCount: 0,
    trayReady: false,
    storageReady: false,
    extensionDirectory: ""
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
  pinnedModules: [],
  extensions: [],
  commands: []
};

export const appState = writable(initialState);
export const query = writable("");
export const activeCategoryFilter = writable<CommandCategory | "all">("all");
export const selectedCommandId = writable<string | null>(null);
export const commandInput = writable(defaultCommandInput("data.format-json"));
export const commandResult = writable<CommandExecutionResult | null>(null);
export const commandError = writable<string | null>(null);
export const commandRunning = writable(false);
export const previewRunning = writable(false);
export const historyQuery = writable("");
export const historyResults = writable<CommandHistoryEntry[]>([]);
export const historyLoading = writable(false);

export const filteredCommands = derived(
  [appState, query, activeCategoryFilter],
  ([$appState, $query, $activeCategoryFilter]) =>
    rankCommands(
      visibleCommands($appState).filter((command) =>
        $activeCategoryFilter === "all" ? true : command.category === $activeCategoryFilter
      ),
      $appState.commandUsage,
      $query
    )
);

export const recentCommands = derived(appState, ($appState) => {
  const unique = new Set<string>();
  const recent = [];
  for (const item of $appState.recentHistory) {
    if (unique.has(item.commandId)) continue;
    const command = $appState.commands.find((entry) => entry.id === item.commandId);
    if (!command) continue;
    unique.add(item.commandId);
    recent.push(command);
    if (recent.length >= 8) break;
  }
  return recent;
});

export const selectedCommand = derived(
  [filteredCommands, selectedCommandId],
  ([$filteredCommands, $selectedCommandId]) =>
    $filteredCommands.find((command) => command.id === $selectedCommandId) ?? $filteredCommands[0] ?? null
);

let previewTimeout: ReturnType<typeof setTimeout> | null = null;
let previewRequestId = 0;

export function loadBootstrap(payload: BootstrapPayload) {
  appState.set(payload);
  applyThemeMode(payload.settings.themeMode);
  const initialCommand =
    visibleCommands(payload).find((command) => command.id === payload.health.profile.defaultCommandId) ??
    visibleCommands(payload)[0] ??
    payload.commands[0] ??
    null;
  selectedCommandId.set(initialCommand?.id ?? null);
  commandInput.set(defaultCommandInput(initialCommand?.id));
  historyResults.set(payload.recentHistory);
  scheduleLivePreview(initialCommand, defaultCommandInput(initialCommand?.id));
}

export function refreshBootstrap(payload: BootstrapPayload) {
  const currentSelection = get(selectedCommandId);
  const currentInput = get(commandInput);

  appState.set(payload);
  applyThemeMode(payload.settings.themeMode);

  const nextCommand =
    payload.commands.find((command) => command.id === currentSelection) ??
    visibleCommands(payload).find((command) => command.id === payload.health.profile.defaultCommandId) ??
    visibleCommands(payload)[0] ??
    payload.commands[0] ??
    null;

  selectedCommandId.set(nextCommand?.id ?? null);
  commandInput.set(nextCommand?.id === currentSelection ? currentInput : defaultCommandInput(nextCommand?.id));
  historyResults.set(get(historyQuery).trim() ? get(historyResults) : payload.recentHistory);
}

export async function executeSelectedCommand(command: CommandAction | null, currentQuery: string) {
  if (!command) return;

  if (previewTimeout) {
    clearTimeout(previewTimeout);
  }
  previewRequestId += 1;
  previewRunning.set(false);
  commandRunning.set(true);
  commandError.set(null);

  try {
    const input = get(commandInput);
    const result = await executeCommand(command.id, input);
    commandResult.set(result);
    const historyQuery = currentQuery.trim() || summarizeExecutionInput(input, command.id);
    const recentHistory = await recordCommandExecution(command.id, historyQuery, input);
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
          inputText: get(commandInput),
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
    applyThemeMode(saved.themeMode);
    appState.update((state) => ({ ...state, settings: saved }));
  } catch {
    applyThemeMode(settings.themeMode);
    appState.update((state) => ({ ...state, settings }));
  }
}

export async function activateProfile(profileId: string) {
  const payload = await setActiveProfile(profileId);
  loadBootstrap(payload);
}

export async function persistWorkspaceProfile(payload: SaveWorkspaceProfilePayload) {
  const refreshed = await saveWorkspaceProfile(payload);
  loadBootstrap(refreshed);
}

export function selectCommand(command: CommandAction) {
  selectCommandById(command.id);
}

export function setCategoryFilter(category: CommandCategory | "all") {
  activeCategoryFilter.set(category);
}

export function selectCommandById(commandId: string, nextInput?: string) {
  const command = get(appState).commands.find((item) => item.id === commandId) ?? null;
  selectedCommandId.set(commandId);
  const resolvedInput = nextInput ?? defaultCommandInput(commandId);
  commandInput.set(resolvedInput);
  commandResult.set(null);
  commandError.set(null);
  scheduleLivePreview(command, resolvedInput);
}

export async function rerunHistoryEntry(entry: CommandHistoryEntry) {
  const command = get(appState).commands.find((item) => item.id === entry.commandId) ?? null;
  if (!command) return;

  selectCommandById(entry.commandId, entry.inputText);
  await executeSelectedCommand(command, entry.queryText);
}

export async function searchHistory(queryText: string) {
  historyQuery.set(queryText);
  historyLoading.set(true);
  try {
    if (!queryText.trim()) {
      historyResults.set(get(appState).recentHistory);
      return;
    }

    historyResults.set(await searchCommandHistory(queryText, 50));
  } finally {
    historyLoading.set(false);
  }
}

export async function togglePinnedCommand(command: CommandAction | null) {
  if (!command) return;
  const payload = await togglePinnedModule(command.id);
  loadBootstrap(payload);
}

export async function refreshExtensions() {
  const payload = await reloadExtensions();
  loadBootstrap(payload);
}

export async function exportConfigSnapshot() {
  return exportConfigurationSnapshot();
}

export async function importConfigSnapshot(snapshot: ConfigurationSnapshot) {
  const payload = await importConfiguration(snapshot);
  loadBootstrap(payload);
}

export async function readConfigFile(path: string) {
  return readTextFile(path);
}

export function isCommandPinned(commandId: string | null) {
  if (!commandId) return false;
  return get(appState).pinnedModules.some((entry) => entry.commandId === commandId);
}

export function setThemeMode(themeMode: string) {
  const current = get(appState).settings;
  return saveSettings({ ...current, themeMode });
}

export function applyThemeMode(themeMode: string) {
  const root = document.documentElement;
  const systemDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;
  const systemContrast = window.matchMedia?.("(prefers-contrast: more)").matches ?? false;
  const resolved =
    themeMode === "system"
      ? systemContrast
        ? "contrast"
        : systemDark
          ? "dark"
          : "light"
      : themeMode;

  root.dataset.theme = resolved;
  root.style.colorScheme = resolved === "light" ? "light" : "dark";
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

function visibleCommands(state: BootstrapPayload) {
  const enabled = new Set(state.health.profile.enabledCommandIds);
  if (enabled.size === 0) {
    return state.commands;
  }

  return state.commands.filter((command) => enabled.has(command.id));
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

function scheduleLivePreview(command: CommandAction | null, input: string) {
  if (previewTimeout) {
    clearTimeout(previewTimeout);
  }

  if (!command?.acceptsInput || get(commandRunning) || !getToolSpec(command).livePreview) {
    previewRunning.set(false);
    return;
  }

  const requestId = ++previewRequestId;
  previewRunning.set(true);

  previewTimeout = setTimeout(() => {
    void runLivePreview(command, input, requestId);
  }, 120);
}

async function runLivePreview(command: CommandAction, input: string, requestId: number) {
  try {
    const result = await executeCommand(command.id, input);
    if (requestId !== previewRequestId || get(selectedCommandId) !== command.id) {
      return;
    }

    commandResult.set(result);
    commandError.set(null);
  } catch (error) {
    if (requestId !== previewRequestId || get(selectedCommandId) !== command.id) {
      return;
    }

    const message = error instanceof Error ? error.message : "Preview failed.";
    commandError.set(message);
  } finally {
    if (requestId === previewRequestId) {
      previewRunning.set(false);
    }
  }
}
export function updateCommandInput(value: string) {
  commandInput.set(value);
  scheduleLivePreview(get(selectedCommand), value);
}

export function applyCommandTemplate(value: string) {
  commandInput.set(value);
  scheduleLivePreview(get(selectedCommand), value);
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
