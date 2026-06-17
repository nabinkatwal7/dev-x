<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import {
    commandError,
    commandInput,
    commandResult,
    commandRunning,
    executeSelectedCommand,
    query,
    saveSettings,
    updateCommandInput
  } from "../stores/app-shell";
  import type {
    AppHealth,
    AppSettings,
    CommandAction,
    CommandHistoryEntry
  } from "../types";

  export let command: CommandAction | null;
  export let health: AppHealth;
  export let settings: AppSettings;
  export let recentHistory: CommandHistoryEntry[];

  async function toggleTrayPreference() {
    await saveSettings({
      ...settings,
      closeToTray: !settings.closeToTray
    });
  }

  async function runCommand() {
    await executeSelectedCommand(command, $query);
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
    <div class="min-w-0 rounded-md border border-chrome-700 bg-chrome-950 p-3">
      <div class="flex items-center justify-between">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Command Workspace</div>
        {#if command}
          <button
            class="rounded border border-accent-400/70 px-2 py-1 text-[11px] text-chrome-100 transition hover:bg-accent-500/15 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={$commandRunning}
            on:click={runCommand}
          >
            {$commandRunning ? "Running" : "Run"}
          </button>
        {/if}
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
              />
            </div>
          {/if}

          <div class="grid gap-2">
            <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Result</div>
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
        <p class="mt-3 text-sm text-chrome-300">Select a command to inspect the first architecture slice.</p>
      {/if}
    </div>

    <div class="min-w-0 space-y-3">
      <div class="rounded-md border border-chrome-700 bg-chrome-950 p-3">
        <div class="text-[11px] uppercase tracking-[0.18em] text-chrome-300">Application</div>
        <dl class="mt-3 grid gap-2 text-sm">
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Workspace Profile</dt>
            <dd class="text-chrome-100">{health.profile.name}</dd>
          </div>
          <div class="flex items-center justify-between">
            <dt class="text-chrome-300">Storage Layer</dt>
            <dd class:text-signal-success={health.storageReady} class="text-chrome-100">
              {health.storageReady ? "Ready" : "Unavailable"}
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
    </div>
  </div>
</section>
