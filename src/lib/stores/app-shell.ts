import { derived, writable } from "svelte/store";
import type { BootstrapPayload, CommandAction } from "../types";

const initialState: BootstrapPayload = {
  health: {
    profile: {
      id: "loading",
      name: "Loading",
      enabledCategories: ["core"]
    },
    commandCount: 0,
    trayReady: false,
    storageReady: false
  },
  commands: []
};

export const appState = writable(initialState);
export const query = writable("");
export const selectedCommandId = writable<string | null>(null);

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
