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

<main class="min-h-screen px-6 py-8 text-chrome-100">
  <section class="mx-auto flex min-h-[calc(100vh-4rem)] max-w-7xl flex-col gap-5">
    <header class="flex flex-col justify-between gap-4 rounded-lg border border-chrome-700/70 bg-chrome-900/55 px-6 py-5 backdrop-blur lg:flex-row lg:items-end">
      <div>
        <div class="text-xs uppercase tracking-[0.24em] text-chrome-300">DevForge</div>
        <h1 class="mt-2 text-4xl font-semibold text-chrome-100">Local desktop developer supertool</h1>
        <p class="mt-3 max-w-3xl text-sm leading-6 text-chrome-300">
          Phase 1 architecture scaffold for a Tauri + Rust + Svelte application centered on the launcher,
          typed IPC, and modular local services.
        </p>
      </div>

      <div class="grid gap-2 rounded-md border border-chrome-700 bg-chrome-950/70 px-4 py-3 text-sm text-chrome-200">
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

    <section class="grid flex-1 gap-5 lg:grid-cols-[420px_minmax(0,1fr)]">
      <CommandPalette commandCount={$appState.health.commandCount} />
      <PreviewPane command={$selectedCommand} health={$appState.health} />
    </section>
  </section>
</main>
