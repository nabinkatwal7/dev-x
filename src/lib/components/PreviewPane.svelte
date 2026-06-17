<script lang="ts">
  import { categoryLabels } from "../core/command-sections";
  import type { AppHealth, CommandAction } from "../types";

  export let command: CommandAction | null;
  export let health: AppHealth;

  const roadmap = [
    "Launcher overlay lifecycle and hotkey wiring",
    "Local command registry with profile-aware filtering",
    "SQLite-backed action history and settings store",
    "Pluggable Rust service boundaries for utilities"
  ];
</script>

<section class="flex h-full flex-col rounded-lg border border-chrome-700/70 bg-chrome-900/55 shadow-overlay backdrop-blur">
  <div class="border-b border-chrome-700/70 px-5 py-4">
    <div class="text-xs uppercase tracking-[0.18em] text-chrome-300">Preview</div>
    <h2 class="mt-2 text-2xl font-semibold text-chrome-100">
      {command?.title ?? "Waiting for selection"}
    </h2>
    <p class="mt-2 max-w-xl text-sm text-chrome-300">
      {command?.subtitle ?? "Core architecture placeholder for live command previews and module-specific interactions."}
    </p>
  </div>

  <div class="grid flex-1 gap-4 p-5 lg:grid-cols-[1.2fr_0.8fr]">
    <div class="rounded-md border border-chrome-700 bg-chrome-950/70 p-4">
      <div class="text-xs uppercase tracking-[0.18em] text-chrome-300">Command Metadata</div>
      {#if command}
        <dl class="mt-4 grid gap-4 text-sm">
          <div>
            <dt class="text-chrome-300">Category</dt>
            <dd class="mt-1 font-semibold text-chrome-100">{categoryLabels[command.category]}</dd>
          </div>
          <div>
            <dt class="text-chrome-300">Tags</dt>
            <dd class="mt-1 font-mono text-chrome-100">{command.tags.join(", ")}</dd>
          </div>
          <div>
            <dt class="text-chrome-300">Execution Model</dt>
            <dd class="mt-1 text-chrome-100">IPC-routed local command with frontend preview contract.</dd>
          </div>
        </dl>
      {:else}
        <p class="mt-4 text-sm text-chrome-300">Select a command to inspect the first architecture slice.</p>
      {/if}
    </div>

    <div class="space-y-4">
      <div class="rounded-md border border-chrome-700 bg-chrome-950/70 p-4">
        <div class="text-xs uppercase tracking-[0.18em] text-chrome-300">System Health</div>
        <dl class="mt-4 grid gap-3 text-sm">
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
        </dl>
      </div>

      <div class="rounded-md border border-chrome-700 bg-chrome-950/70 p-4">
        <div class="text-xs uppercase tracking-[0.18em] text-chrome-300">Foundation Scope</div>
        <ul class="mt-4 space-y-3 text-sm text-chrome-200">
          {#each roadmap as item}
            <li>{item}</li>
          {/each}
        </ul>
      </div>
    </div>
  </div>
</section>
