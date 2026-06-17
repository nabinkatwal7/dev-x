<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import { decryptConfiguration, encryptConfiguration } from "../core/config-crypto";
  import { writeTextFile } from "../ipc/client";
  import {
    activateProfile,
    appState,
    commandError,
    commandInput,
    commandResult,
    commandRunning,
    executeSelectedCommand,
    exportConfigSnapshot,
    historyLoading,
    historyResults,
    importConfigSnapshot,
    persistWorkspaceProfile,
    previewRunning,
    query,
    readConfigFile,
    refreshExtensions,
    rerunHistoryEntry,
    saveSettings,
    searchHistory,
    setThemeMode,
    togglePinnedCommand,
    updateCommandInput
  } from "../stores/app-shell";
  import type {
    AppHealth,
    AppSettings,
    CommandAction,
    CommandHistoryEntry,
    PinnedModule,
    ScriptExtensionSummary,
    WorkspaceProfile
  } from "../types";

  export let command: CommandAction | null;
  export let health: AppHealth;
  export let settings: AppSettings;
  export let profiles: WorkspaceProfile[];
  export let commands: CommandAction[];
  export let recentHistory: CommandHistoryEntry[];
  export let pinnedModules: PinnedModule[] = [];
  export let extensions: ScriptExtensionSummary[] = [];
  export let standalone = false;

  let profileName = "";
  let profileTags = "";
  let profileHotkey = "";
  let enabledCommandIds: string[] = [];
  let defaultCommandId = "";
  let historySearchText = "";
  let exportPath = "";
  let importPath = "";
  let configPassphrase = "";
  let syncStatus = "";
  let syncError = "";

  const themeOptions = [
    { value: "system", label: "System" },
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
    { value: "contrast", label: "Contrast" }
  ];

  $: if (health.profile) {
    profileName = health.profile.name;
    profileTags = health.profile.environmentTags.join(", ");
    profileHotkey = health.profile.launchHotkey;
    enabledCommandIds = [...health.profile.enabledCommandIds];
    defaultCommandId = health.profile.defaultCommandId;
  }

  async function toggleTrayPreference() {
    await saveSettings({
      ...settings,
      closeToTray: !settings.closeToTray
    });
  }

  async function runCommand() {
    await executeSelectedCommand(command, $query);
  }

  async function saveProfile() {
    const nextEnabled = enabledCommandIds.length > 0 ? enabledCommandIds : commands.map((item) => item.id);
    const nextDefault =
      nextEnabled.find((commandId) => commandId === defaultCommandId) ?? nextEnabled[0] ?? "";

    await persistWorkspaceProfile({
      id: health.profile.id,
      name: profileName.trim() || "Workspace",
      environmentTags: profileTags
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean),
      enabledCommandIds: nextEnabled,
      defaultCommandId: nextDefault,
      launchHotkey: profileHotkey.trim() || settings.launchHotkey
    });
  }

  async function createProfile() {
    const nextEnabled = enabledCommandIds.length > 0 ? enabledCommandIds : commands.map((item) => item.id);
    await persistWorkspaceProfile({
      name: `${profileName.trim() || "Workspace"} Copy`,
      environmentTags: profileTags
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean),
      enabledCommandIds: nextEnabled,
      defaultCommandId: defaultCommandId || nextEnabled[0] || "",
      launchHotkey: profileHotkey.trim() || settings.launchHotkey
    });
  }

  function toggleCommand(commandId: string, checked: boolean) {
    enabledCommandIds = checked
      ? [...new Set([...enabledCommandIds, commandId])]
      : enabledCommandIds.filter((id) => id !== commandId);

    if (!enabledCommandIds.includes(defaultCommandId)) {
      defaultCommandId = enabledCommandIds[0] ?? "";
    }
  }

  async function exportConfiguration() {
    syncStatus = "";
    syncError = "";

    try {
      const snapshot = await exportConfigSnapshot();
      const encrypted = await encryptConfiguration(snapshot, configPassphrase);
      await writeTextFile(exportPath, encrypted);
      syncStatus = `Exported ${snapshot.profiles.length} profiles to ${exportPath}.`;
    } catch (error) {
      syncError = error instanceof Error ? error.message : "Export failed.";
    }
  }

  async function importConfiguration() {
    syncStatus = "";
    syncError = "";

    try {
      const encrypted = await readConfigFile(importPath);
      const snapshot = await decryptConfiguration(encrypted, configPassphrase);
      await importConfigSnapshot(snapshot);
      syncStatus = `Imported ${snapshot.profiles.length} profiles from ${importPath}.`;
    } catch (error) {
      syncError = error instanceof Error ? error.message : "Import failed.";
    }
  }
</script>

<section class="flex h-full flex-col rounded-lg border border-chrome-700/70 bg-chrome-900 shadow-overlay">
  <div class="border-b border-chrome-700/70 px-4 py-3">
    <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">
      {standalone ? "Pinned Module" : "Preview"}
    </div>
    <h2 class="mt-1 text-xl font-semibold text-chrome-100">
      {command?.title ?? "Waiting for selection"}
    </h2>
    <p class="mt-1.5 max-w-xl text-sm leading-5 text-chrome-300">
      {command?.subtitle ?? "Select a working module from the omnibar."}
    </p>
  </div>

  {#if standalone}
    <div class="min-h-0 flex-1 p-3">
      <div class="h-full min-w-0 rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Workspace</div>
          <button
            class="rounded border border-accent-400/70 px-2 py-1 text-[11px] text-chrome-100 transition hover:bg-accent-500/15 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={$commandRunning}
            on:click={runCommand}
          >
            {$commandRunning ? "Running" : "Run"}
          </button>
        </div>

        {#if command?.acceptsInput}
          <div class="mt-3 grid gap-2">
            <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="command-input">
              Input
            </label>
            <textarea
              id="command-input"
              class="min-h-[220px] w-full resize-y rounded-md border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm leading-5 text-chrome-100 outline-none transition focus:border-accent-400"
              bind:value={$commandInput}
              on:input={(event) => updateCommandInput((event.currentTarget as HTMLTextAreaElement).value)}
            ></textarea>
          </div>
        {/if}

        <div class="mt-3 grid gap-2">
          <div class="flex items-center justify-between">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Result</div>
            <div class="text-[11px] text-chrome-300">
              {command?.acceptsInput ? ($previewRunning ? "Previewing" : "Live preview") : ""}
            </div>
          </div>
          {#if $commandError}
            <div class="rounded-md border border-signal-danger/50 bg-chrome-900 px-3 py-2 text-sm text-signal-danger">
              {$commandError}
            </div>
          {:else if $commandResult}
            <div class="rounded-md border border-chrome-700 bg-chrome-900 px-3 py-2">
              <div class="text-sm font-semibold text-chrome-100">{$commandResult.title}</div>
              <div class="mt-1 text-sm text-chrome-300">{$commandResult.summary}</div>
              {#if $commandResult.output}
                <pre class="mt-3 overflow-x-hidden whitespace-pre-wrap break-all rounded-md border border-chrome-700 bg-chrome-950 p-3 text-xs leading-5 text-chrome-100">{$commandResult.output}</pre>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <div class="grid min-h-0 flex-1 gap-3 p-3 lg:grid-cols-[1.15fr_0.85fr]">
      <div class="min-w-0 rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Command Workspace</div>
          <div class="flex items-center gap-2">
            {#if command?.acceptsInput}
              <span class="text-[11px] text-chrome-300">
                {$previewRunning ? "Previewing" : "Live preview"}
              </span>
            {/if}
            {#if command}
              <button
                class="rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
                on:click={() => togglePinnedCommand(command)}
              >
                {pinnedModules.some((item) => item.commandId === command.id) ? "Unpin Window" : "Pin Window"}
              </button>
              <button
                class="rounded border border-accent-400/70 px-2 py-1 text-[11px] text-chrome-100 transition hover:bg-accent-500/15 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={$commandRunning}
                on:click={runCommand}
              >
                {$commandRunning ? "Running" : "Run"}
              </button>
            {/if}
          </div>
        </div>

        {#if command}
          <div class="mt-3 grid gap-3">
            <dl class="grid gap-3 text-sm">
              <div>
                <dt class="text-chrome-300">Category</dt>
                <dd class="mt-0.5 font-semibold text-chrome-100">{categoryLabels[command.category]}</dd>
              </div>
              <div>
                <dt class="text-chrome-300">Tags</dt>
                <dd class="mt-0.5 font-mono text-chrome-100">{command.tags.join(", ")}</dd>
              </div>
            </dl>

            {#if command.acceptsInput}
              <div class="grid gap-2">
                <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="command-input">
                  Input
                </label>
                <textarea
                  id="command-input"
                  class="min-h-[260px] w-full resize-y rounded-md border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm leading-5 text-chrome-100 outline-none transition focus:border-accent-400"
                  bind:value={$commandInput}
                  on:input={(event) => updateCommandInput((event.currentTarget as HTMLTextAreaElement).value)}
                ></textarea>
              </div>
            {/if}

            <div class="grid gap-2">
              <div class="flex items-center justify-between">
                <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Result</div>
                {#if command.acceptsInput}
                  <div class="text-[11px] text-chrome-300">Updates as you type</div>
                {/if}
              </div>
              {#if $commandError}
                <div class="rounded-md border border-signal-danger/50 bg-chrome-900 px-3 py-2 text-sm text-signal-danger">
                  {$commandError}
                </div>
              {:else if $commandResult}
                <div class="rounded-md border border-chrome-700 bg-chrome-900 px-3 py-2">
                  <div class="text-sm font-semibold text-chrome-100">{$commandResult.title}</div>
                  <div class="mt-1 text-sm text-chrome-300">{$commandResult.summary}</div>
                  {#if $commandResult.output}
                    <pre class="mt-3 overflow-x-hidden whitespace-pre-wrap break-all rounded-md border border-chrome-700 bg-chrome-950 p-3 text-xs leading-5 text-chrome-100">{$commandResult.output}</pre>
                  {/if}
                </div>
              {:else}
                <div class="rounded-md border border-dashed border-chrome-700 px-3 py-3 text-sm text-chrome-300">
                  {command.acceptsInput
                    ? "Provide input and run the command."
                    : "Run the command to see its result."}
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <p class="mt-3 text-sm text-chrome-300">Select a command to inspect the first working slice.</p>
        {/if}
      </div>

      <div class="min-w-0 space-y-3 overflow-y-auto pr-1">
        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Application</div>
          <dl class="mt-3 grid gap-2 text-sm">
            <div class="flex items-center justify-between gap-3">
              <dt class="text-chrome-300">Workspace Profile</dt>
              <dd class="text-right text-chrome-100">{health.profile.name}</dd>
            </div>
            <div class="flex items-center justify-between gap-3">
              <dt class="text-chrome-300">Environment Tags</dt>
              <dd class="text-right text-chrome-100">{health.profile.environmentTags.join(", ") || "None"}</dd>
            </div>
            <div class="flex items-center justify-between gap-3">
              <dt class="text-chrome-300">Pinned Windows</dt>
              <dd class="text-right text-chrome-100">{pinnedModules.length}</dd>
            </div>
            <div class="flex items-center justify-between gap-3">
              <dt class="text-chrome-300">Extension Directory</dt>
              <dd class="max-w-[18rem] break-all text-right text-chrome-100">{health.extensionDirectory}</dd>
            </div>
          </dl>
        </div>

        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="flex items-center justify-between gap-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Theme Matrix</div>
            <select
              class="rounded border border-chrome-700 bg-chrome-900 px-2 py-1 text-[11px] text-chrome-100"
              value={settings.themeMode}
              on:change={(event) => setThemeMode((event.currentTarget as HTMLSelectElement).value)}
            >
              {#each themeOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>
          <div class="mt-3 grid gap-2 text-sm">
            <div class="flex items-center justify-between gap-3">
              <span class="text-chrome-300">Resolved Mode</span>
              <span class="text-chrome-100">{document.documentElement.dataset.theme ?? settings.themeMode}</span>
            </div>
            <div class="flex items-center justify-between gap-3">
              <span class="text-chrome-300">Close To Tray</span>
              <button
                class="rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
                on:click={toggleTrayPreference}
              >
                {settings.closeToTray ? "Tray On" : "Tray Off"}
              </button>
            </div>
          </div>
        </div>

        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="flex items-center justify-between">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Workspace Profiles</div>
            <select
              class="rounded border border-chrome-700 bg-chrome-900 px-2 py-1 text-[11px] text-chrome-100"
              value={health.profile.id}
              on:change={(event) => activateProfile((event.currentTarget as HTMLSelectElement).value)}
            >
              {#each profiles as profile}
                <option value={profile.id}>{profile.name}</option>
              {/each}
            </select>
          </div>

          <div class="mt-3 grid gap-3">
            <div class="grid gap-1">
              <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="profile-name">Name</label>
              <input
                id="profile-name"
                class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
                bind:value={profileName}
              />
            </div>

            <div class="grid gap-1">
              <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="profile-tags">Environment Tags</label>
              <input
                id="profile-tags"
                class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
                bind:value={profileTags}
                placeholder="frontend, devops, security"
              />
            </div>

            <div class="grid gap-1">
              <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="profile-hotkey">Overlay Hotkey</label>
              <input
                id="profile-hotkey"
                class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
                bind:value={profileHotkey}
                placeholder="Alt+Space"
              />
            </div>

            <div class="grid gap-2">
              <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Enabled Tools</div>
              {#each commands as availableCommand}
                <label class="flex items-center justify-between gap-3 rounded border border-chrome-800 bg-chrome-900 px-3 py-2 text-sm text-chrome-100">
                  <span class="min-w-0 truncate">{availableCommand.title}</span>
                  <input
                    type="checkbox"
                    checked={enabledCommandIds.includes(availableCommand.id)}
                    on:change={(event) =>
                      toggleCommand(availableCommand.id, (event.currentTarget as HTMLInputElement).checked)}
                  />
                </label>
              {/each}
            </div>

            <div class="grid gap-1">
              <label class="text-[11px] uppercase tracking-[0.18em] text-chrome-300" for="default-command">Default Tool</label>
              <select
                id="default-command"
                class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
                bind:value={defaultCommandId}
              >
                {#each commands.filter((item) => enabledCommandIds.includes(item.id)) as availableCommand}
                  <option value={availableCommand.id}>{availableCommand.title}</option>
                {/each}
              </select>
            </div>

            <div class="flex gap-2">
              <button
                class="rounded border border-accent-400/70 px-3 py-2 text-xs text-chrome-100 transition hover:bg-accent-500/15"
                on:click={saveProfile}
              >
                Save Active
              </button>
              <button
                class="rounded border border-chrome-600 px-3 py-2 text-xs text-chrome-100 transition hover:border-accent-400"
                on:click={createProfile}
              >
                Save As New
              </button>
            </div>
          </div>
        </div>

        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="flex items-center justify-between gap-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Action History</div>
            <div class="text-[11px] text-chrome-300">
              {$historyLoading ? "Searching" : historySearchText.trim() ? $historyResults.length : recentHistory.length}
            </div>
          </div>
          <div class="mt-3 grid gap-2">
            <input
              class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
              bind:value={historySearchText}
              placeholder="Search command id, query, or input"
              on:input={() => searchHistory(historySearchText)}
            />
            <ul class="space-y-2 text-sm text-chrome-200">
              {#if $historyLoading}
                <li class="text-chrome-300">Searching history...</li>
              {:else if $historyResults.length === 0}
                <li class="text-chrome-300">No matching executions.</li>
              {:else}
                {#each $historyResults as item}
                  <li class="rounded border border-chrome-800 bg-chrome-900 p-2">
                    <div class="flex items-start justify-between gap-3">
                      <div class="min-w-0">
                        <div class="font-mono text-chrome-100">{item.commandId}</div>
                        <div class="truncate text-chrome-300">{item.queryText || "No query text"}</div>
                        <div class="mt-1 text-[11px] uppercase tracking-[0.18em] text-chrome-300">{item.executedAt}</div>
                      </div>
                      <button
                        class="shrink-0 rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
                        on:click={() => rerunHistoryEntry(item)}
                      >
                        Re-run
                      </button>
                    </div>
                  </li>
                {/each}
              {/if}
            </ul>
          </div>
        </div>

        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="flex items-center justify-between gap-3">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Extension Loader</div>
            <button
              class="rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
              on:click={refreshExtensions}
            >
              Reload
            </button>
          </div>
          <div class="mt-3 text-xs text-chrome-300 break-all">{health.extensionDirectory}</div>
          <ul class="mt-3 space-y-2 text-sm text-chrome-200">
            {#if extensions.length === 0}
              <li class="text-chrome-300">Drop manifest `.json` files into the extension directory.</li>
            {:else}
              {#each extensions as extension}
                <li class="rounded border border-chrome-800 bg-chrome-900 p-2">
                  <div class="font-semibold text-chrome-100">{extension.title}</div>
                  <div class="text-chrome-300">{extension.subtitle}</div>
                  <div class="mt-1 break-all text-[11px] text-chrome-300">{extension.commandPath}</div>
                </li>
              {/each}
            {/if}
          </ul>
        </div>

        <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
          <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Config Sync</div>
          <div class="mt-3 grid gap-2">
            <input
              class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
              bind:value={exportPath}
              placeholder="Export file path"
            />
            <input
              class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
              bind:value={importPath}
              placeholder="Import file path"
            />
            <input
              class="rounded border border-chrome-700 bg-chrome-900 px-3 py-2 text-sm text-chrome-100 outline-none focus:border-accent-400"
              bind:value={configPassphrase}
              placeholder="Passphrase"
              type="password"
            />
            <div class="flex gap-2">
              <button
                class="rounded border border-accent-400/70 px-3 py-2 text-xs text-chrome-100 transition hover:bg-accent-500/15"
                disabled={!exportPath.trim() || !configPassphrase.trim()}
                on:click={exportConfiguration}
              >
                Export
              </button>
              <button
                class="rounded border border-chrome-600 px-3 py-2 text-xs text-chrome-100 transition hover:border-accent-400"
                disabled={!importPath.trim() || !configPassphrase.trim()}
                on:click={importConfiguration}
              >
                Import
              </button>
            </div>
            {#if syncStatus}
              <div class="text-sm text-signal-success">{syncStatus}</div>
            {/if}
            {#if syncError}
              <div class="text-sm text-signal-danger">{syncError}</div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</section>
