<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import { activateProfile, saveSettings } from "../stores/app-shell";
  import type { AppHealth, AppSettings, CommandAction, CommandHistoryEntry, WorkspaceProfile } from "../types";

  export let command: CommandAction | null;
  export let health: AppHealth;
  export let settings: AppSettings;
  export let profiles: WorkspaceProfile[];
  export let recentHistory: CommandHistoryEntry[];

  const roadmap = [
    "Launcher overlay lifecycle and hotkey wiring",
    "Local command registry with profile-aware filtering",
    "SQLite-backed action history and settings store",
    "Pluggable Rust service boundaries for utilities"
  ];

  async function toggleTrayPreference() {
    await saveSettings({
      ...settings,
      closeToTray: !settings.closeToTray
    });
  }
</script>

<section class="flex h-full flex-col rounded-lg border border-chrome-700/70 bg-chrome-900 shadow-overlay">
  <div class="border-b border-chrome-700/70 px-4 py-3">
    <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Preview</div>
    <h2 class="mt-1 text-xl font-semibold text-chrome-100">
      {command?.title ?? "Waiting for selection"}
    </h2>
    <p class="mt-1.5 max-w-xl text-sm leading-5 text-chrome-300">
      {command?.subtitle ?? "Core architecture placeholder for live command previews and module-specific interactions."}
    </p>
  </div>

  <div class="grid flex-1 gap-3 p-3 lg:grid-cols-[1.15fr_0.85fr]">
    <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
      <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Command Metadata</div>
      {#if command}
        <dl class="mt-3 grid gap-3 text-sm">
          <div>
            <dt class="text-chrome-300">Category</dt>
            <dd class="mt-0.5 font-semibold text-chrome-100">{categoryLabels[command.category]}</dd>
          </div>
          <div>
            <dt class="text-chrome-300">Tags</dt>
            <dd class="mt-0.5 font-mono text-chrome-100">{command.tags.join(", ")}</dd>
          </div>
          <div>
            <dt class="text-chrome-300">Execution Model</dt>
            <dd class="mt-0.5 text-chrome-100">IPC-routed local command with frontend preview contract.</dd>
          </div>
        </dl>
      {:else}
        <p class="mt-3 text-sm text-chrome-300">Select a command to inspect the first architecture slice.</p>
      {/if}
    </div>

    <div class="space-y-3">
      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">System Health</div>
        <dl class="mt-3 grid gap-2 text-sm">
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Workspace Profile</dt>
            <dd class="text-chrome-100">{health.profile.name}</dd>
          </div>
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Tray Agent</dt>
            <dd class:text-signal-success={health.trayReady} class:text-signal-warning={!health.trayReady}>
              {health.trayReady ? "Ready" : "Scaffolded"}
            </dd>
          </div>
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Storage Layer</dt>
            <dd class:text-signal-success={health.storageReady} class:text-signal-warning={!health.storageReady}>
              {health.storageReady ? "Ready" : "Pending"}
            </dd>
          </div>
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Theme Mode</dt>
            <dd class="text-chrome-100">{settings.themeMode}</dd>
          </div>
        </dl>
      </div>

      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="flex items-center justify-between">
          <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Persistent Settings</div>
          <button
            class="rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
            on:click={toggleTrayPreference}
          >
            {settings.closeToTray ? "Tray On" : "Tray Off"}
          </button>
        </div>
        <dl class="mt-3 grid gap-2 text-sm">
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Launch Hotkey</dt>
            <dd class="font-mono text-chrome-100">{settings.launchHotkey}</dd>
          </div>
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">History Limit</dt>
            <dd class="text-chrome-100">{settings.historyLimit}</dd>
          </div>
        </dl>
      </div>

      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Workspace Profiles</div>
        <ul class="mt-3 space-y-2 text-sm">
          {#each profiles as profile}
            <li class="flex items-center justify-between gap-3">
              <div>
                <div class="text-chrome-100">{profile.name}</div>
                <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">
                  {profile.enabledCategories.join(", ")}
                </div>
              </div>
              {#if profile.id === health.profile.id}
                <span class="rounded border border-accent-400/70 px-2 py-1 text-[11px] text-chrome-100">Active</span>
              {:else}
                <button
                  class="rounded border border-chrome-600 px-2 py-1 text-[11px] text-chrome-100 transition hover:border-accent-400"
                  on:click={() => activateProfile(profile)}
                >
                  Activate
                </button>
              {/if}
            </li>
          {/each}
        </ul>
      </div>

      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Recent Command History</div>
        <ul class="mt-3 space-y-2 text-sm text-chrome-200">
          {#if recentHistory.length === 0}
            <li class="text-chrome-300">No command executions recorded yet.</li>
          {:else}
            {#each recentHistory as item}
              <li>
                <div class="font-mono text-chrome-100">{item.commandId}</div>
                <div class="text-chrome-300">{item.queryText || "No query text"}</div>
                <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">{item.executedAt}</div>
              </li>
            {/each}
          {/if}
        </ul>
      </div>

      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Foundation Scope</div>
        <ul class="mt-3 space-y-2 text-sm text-chrome-200">
          {#each roadmap as item}
            <li>{item}</li>
          {/each}
        </ul>
      </div>
    </div>
  </div>
</section>
