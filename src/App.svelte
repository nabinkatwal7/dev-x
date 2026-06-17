<script lang="ts">
  import { onMount } from "svelte";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import PreviewPane from "./lib/components/PreviewPane.svelte";
  import { bootstrapApp } from "./lib/ipc/client";
  import { appState, loadBootstrap, selectedCommand } from "./lib/stores/app-shell";

  onMount(async () => {
    const payload = await bootstrapApp();
    loadBootstrap(payload);
  });
</script>

<svelte:head>
  <title>DevForge</title>
</svelte:head>

<main class="min-h-screen px-3 py-3 text-chrome-100">
  <section class="flex min-h-[calc(100vh-1.5rem)] flex-col gap-3">
    <header class="flex flex-col justify-between gap-3 rounded-lg border border-chrome-700/70 bg-chrome-900 px-4 py-3 lg:flex-row lg:items-center">
      <div>
        <div class="text-xs uppercase tracking-[0.24em] text-chrome-300">DevForge</div>
        <h1 class="mt-1 text-2xl font-semibold text-chrome-100">Local desktop developer supertool</h1>
        <p class="mt-2 max-w-3xl text-sm leading-5 text-chrome-300">
          Phase 1 scaffold for a Tauri + Rust + Svelte application centered on the launcher,
          typed IPC, and modular local services.
        </p>
      </div>

      <div class="grid gap-1 rounded-md border border-chrome-700 bg-chrome-950 px-3 py-2 text-sm text-chrome-200">
        <div class="flex items-center justify-between gap-8">
          <span>Profile</span>
          <strong>{$appState.health.profile.name}</strong>
        </div>
        <div class="flex items-center justify-between gap-8">
          <span>Commands</span>
          <strong>{$appState.health.commandCount}</strong>
        </div>
      </div>
    </header>

    <section class="grid flex-1 gap-3 lg:grid-cols-[400px_minmax(0,1fr)]">
      <CommandPalette commandCount={$appState.health.commandCount} />
      <PreviewPane
        command={$selectedCommand}
        health={$appState.health}
        settings={$appState.settings}
        profiles={$appState.profiles}
        recentHistory={$appState.recentHistory}
      />
    </section>
  </section>
</main>
