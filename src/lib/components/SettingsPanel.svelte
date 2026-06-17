<script lang="ts">
  import { decryptConfiguration, encryptConfiguration } from "../core/config-crypto";
  import { writeTextFile } from "../ipc/client";
  import {
    activateProfile,
    appState,
    exportConfigSnapshot,
    historyLoading,
    historyResults,
    importConfigSnapshot,
    persistWorkspaceProfile,
    query,
    readConfigFile,
    refreshExtensions,
    rerunHistoryEntry,
    saveSettings,
    searchHistory,
    setThemeMode,
  } from "../stores/app-shell";
  import type {
    AppHealth,
    AppSettings,
    CommandAction,
    CommandHistoryEntry,
    ScriptExtensionSummary,
    WorkspaceProfile
  } from "../types";

  export let health: AppHealth;
  export let settings: AppSettings;
  export let profiles: WorkspaceProfile[];
  export let commands: CommandAction[];
  export let recentHistory: CommandHistoryEntry[];
  export let extensions: ScriptExtensionSummary[] = [];

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

<section class="flex min-h-0 flex-1 flex-col rounded-xl bg-chrome-800">
  <div class="px-5 py-3">
    <h2 class="text-sm font-semibold text-chrome-100">Workspace Configuration</h2>
    <p class="mt-0.5 text-xs text-chrome-400">Manage profiles, themes, extensions, and data synchronization.</p>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto px-5 pb-4">
    <div class="mx-auto max-w-3xl space-y-3">

      <!-- Application -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Application</div>
        <dl class="mt-3 grid gap-2.5 text-sm">
          <div class="flex items-center justify-between gap-3">
            <dt class="text-chrome-400">Workspace Profile</dt>
            <dd class="text-right text-chrome-100">{health.profile.name}</dd>
          </div>
          <div class="flex items-center justify-between gap-3">
            <dt class="text-chrome-400">Environment Tags</dt>
            <dd class="text-right text-chrome-100">{health.profile.environmentTags.join(", ") || "None"}</dd>
          </div>
          <div class="flex items-center justify-between gap-3">
            <dt class="text-chrome-400">Pinned Windows</dt>
            <dd class="text-right text-chrome-100">0</dd>
          </div>
          <div class="flex items-center justify-between gap-3">
            <dt class="text-chrome-400">Extension Directory</dt>
            <dd class="max-w-[18rem] break-all text-right text-chrome-100">{health.extensionDirectory}</dd>
          </div>
        </dl>
      </div>

      <!-- Theme Matrix -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="flex items-center justify-between gap-3">
          <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Theme</div>
          <select
            class="rounded-md bg-chrome-800 px-3 py-1.5 text-xs text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            value={settings.themeMode}
            on:change={(event) => setThemeMode((event.currentTarget as HTMLSelectElement).value)}
          >
            {#each themeOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </div>
        <div class="mt-3 grid gap-2.5 text-sm">
          <div class="flex items-center justify-between gap-3">
            <span class="text-chrome-400">Resolved Mode</span>
            <span class="text-chrome-100">{document.documentElement.dataset.theme ?? settings.themeMode}</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <span class="text-chrome-400">Close To Tray</span>
            <button
              class="rounded-md bg-chrome-800 px-3 py-1.5 text-xs text-chrome-300 transition hover:text-chrome-100"
              on:click={toggleTrayPreference}
            >
              {settings.closeToTray ? "Enabled" : "Disabled"}
            </button>
          </div>
        </div>
      </div>

      <!-- Workspace Profiles -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Workspace Profiles</div>
          <select
            class="rounded-md bg-chrome-800 px-3 py-1.5 text-xs text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            value={health.profile.id}
            on:change={(event) => activateProfile((event.currentTarget as HTMLSelectElement).value)}
          >
            {#each profiles as profile}
              <option value={profile.id}>{profile.name}</option>
            {/each}
          </select>
        </div>

        <div class="mt-4 grid gap-3">
          <div class="grid gap-1">
            <label class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400" for="profile-name">Name</label>
            <input
              id="profile-name"
              class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
              bind:value={profileName}
            />
          </div>
          <div class="grid gap-1">
            <label class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400" for="profile-tags">Environment Tags</label>
            <input
              id="profile-tags"
              class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
              bind:value={profileTags}
              placeholder="frontend, devops, security"
            />
          </div>
          <div class="grid gap-1">
            <label class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400" for="profile-hotkey">Overlay Hotkey</label>
            <input
              id="profile-hotkey"
              class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
              bind:value={profileHotkey}
              placeholder="Alt+Space"
            />
          </div>

          <div class="grid gap-2">
            <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Enabled Tools</div>
            <div class="max-h-48 overflow-y-auto space-y-1">
              {#each commands as availableCommand}
                <label class="flex items-center justify-between gap-3 rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100">
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
          </div>

          <div class="grid gap-1">
            <label class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400" for="default-command">Default Tool</label>
            <select
              id="default-command"
              class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
              bind:value={defaultCommandId}
            >
              {#each commands.filter((item) => enabledCommandIds.includes(item.id)) as availableCommand}
                <option value={availableCommand.id}>{availableCommand.title}</option>
              {/each}
            </select>
          </div>

          <div class="flex gap-2">
            <button
              class="rounded-md bg-accent-500/15 px-4 py-2 text-xs text-accent-400 transition hover:bg-accent-500/25"
              on:click={saveProfile}
            >
              Save Active
            </button>
            <button
              class="rounded-md bg-chrome-800 px-4 py-2 text-xs text-chrome-300 transition hover:text-chrome-100"
              on:click={createProfile}
            >
              Save As New
            </button>
          </div>
        </div>
      </div>

      <!-- Action History -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="flex items-center justify-between gap-3">
          <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Action History</div>
          <div class="text-[11px] text-chrome-400">
            {$historyLoading ? "Searching" : historySearchText.trim() ? $historyResults.length : recentHistory.length}
          </div>
        </div>
        <div class="mt-3 grid gap-2">
          <input
            class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            bind:value={historySearchText}
            placeholder="Search command id, query, or input"
            on:input={() => searchHistory(historySearchText)}
          />
          <ul class="max-h-48 space-y-2 overflow-y-auto text-sm text-chrome-200">
            {#if $historyLoading}
              <li class="text-chrome-400">Searching history...</li>
            {:else if $historyResults.length === 0}
              <li class="text-chrome-400">No matching executions.</li>
            {:else}
              {#each $historyResults as item}
                <li class="rounded-md bg-chrome-800 p-2.5">
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <div class="font-mono text-sm text-chrome-100">{item.commandId}</div>
                      <div class="truncate text-xs text-chrome-400">{item.queryText || "No query text"}</div>
                      <div class="mt-1 text-[10px] uppercase tracking-[0.12em] text-chrome-400">{item.executedAt}</div>
                    </div>
                    <button
                      class="shrink-0 rounded-md bg-chrome-700/50 px-2 py-1 text-[11px] text-chrome-300 transition hover:text-chrome-100"
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

      <!-- Extension Loader -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="flex items-center justify-between gap-3">
          <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Extensions</div>
          <button
            class="rounded-md bg-chrome-800 px-3 py-1.5 text-xs text-chrome-300 transition hover:text-chrome-100"
            on:click={refreshExtensions}
          >
            Reload
          </button>
        </div>
        <div class="mt-3 text-xs text-chrome-400 break-all">{health.extensionDirectory}</div>
        <ul class="mt-3 space-y-2 text-sm text-chrome-200">
          {#if extensions.length === 0}
            <li class="text-chrome-400">Drop manifest `.json` files into the extension directory.</li>
          {:else}
            {#each extensions as extension}
              <li class="rounded-md bg-chrome-800 p-2.5">
                <div class="font-medium text-chrome-100">{extension.title}</div>
                <div class="mt-0.5 text-xs text-chrome-400">{extension.subtitle}</div>
                <div class="mt-1 break-all text-[10px] text-chrome-400">{extension.commandPath}</div>
              </li>
            {/each}
          {/if}
        </ul>
      </div>

      <!-- Config Sync -->
      <div class="rounded-lg bg-chrome-900 p-4">
        <div class="text-[10px] font-medium uppercase tracking-[0.12em] text-chrome-400">Config Sync</div>
        <div class="mt-3 grid gap-2">
          <input
            class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            bind:value={exportPath}
            placeholder="Export file path"
          />
          <input
            class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            bind:value={importPath}
            placeholder="Import file path"
          />
          <input
            class="rounded-md bg-chrome-800 px-3 py-2 text-sm text-chrome-100 outline-none focus:ring-1 focus:ring-accent-500/50"
            bind:value={configPassphrase}
            placeholder="Passphrase"
            type="password"
          />
          <div class="flex gap-2">
            <button
              class="rounded-md bg-accent-500/15 px-4 py-2 text-xs text-accent-400 transition hover:bg-accent-500/25 disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!exportPath.trim() || !configPassphrase.trim()}
              on:click={exportConfiguration}
            >
              Export
            </button>
            <button
              class="rounded-md bg-chrome-800 px-4 py-2 text-xs text-chrome-300 transition hover:text-chrome-100 disabled:cursor-not-allowed disabled:opacity-40"
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
</section>